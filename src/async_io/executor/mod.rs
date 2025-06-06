use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};
use crate::async_io::task_queue::{Task, TaskQueue};
use crate::async_io::waker_util::waker;
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use polling::Events;
use crate::async_io::reactor::REACTOR;

pub static EXECUTOR: Lazy<Arc<Executor>> = Lazy::new(|| {
    Arc::new(Executor::new())
});

pub fn block_on(f: impl Future<Output = Result<()>> + Send + Sync + 'static) -> Result<()> {
    let executor = EXECUTOR.clone();
    executor.spawn(f);
    executor.run()?;
    Ok(())
}

pub fn spawn(f: impl Future<Output = Result<()>> + Send + Sync + 'static) {
    let executor = EXECUTOR.clone();
    executor.spawn(f)
}

pub struct Executor {
    task_queue: RwLock<TaskQueue>
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    pub fn new() -> Self {
        Self { task_queue: RwLock::new(TaskQueue::new()) }
    }

    pub fn spawn(&self, f: impl Future<Output = Result<()>> + Send + Sync + 'static) {
        self.task_queue.write().unwrap().push(Arc::new(Task { future: RwLock::new(Box::pin(f)) }));
    }

    pub fn run(&self) -> Result<()> {
        loop {
            while let Some(task) = { self.task_queue.write().map_err(|e| anyhow!("{e}"))?.pop() } {
                let sender = self.task_queue
                    .read()
                    .map_err(|e| anyhow!("{e}"))?
                    .sender();
                let waker_task = task.clone();
                let waker = waker(move || {
                    sender.send(waker_task.clone()).unwrap();
                });

                let mut ctx = Context::from_waker(&waker);

                let res = {
                    let mut future = task.future.write()
                        .map_err(|e| anyhow!("{e}"))?;
                    future.as_mut().poll(&mut ctx)
                }; 
                match res {
                    Poll::Ready(res) => res?,
                    Poll::Pending => continue
                }
            }

            {
                if !REACTOR.read()
                    .map_err(|e| anyhow!("{e}"))?
                    .waiting_on_events()
                {
                    return Ok(());    
                }
            }

            self.wait_for_io()?;

            self.task_queue
                .write()
                .map_err(|e| anyhow!("{e}"))?
                .recv();
            
            println!("{:?}", self.task_queue)
        }
    }

    fn wait_for_io(& self) -> Result<()> {
        let mut events = Events::new();
        {
            let reactor = REACTOR.read()
                .map_err(|e| anyhow!("{e}"))?;
            reactor.wait(&mut events, None)?;
        }
        
        let wakers = {
            let mut reactor = REACTOR.write()
                .map_err(|e| anyhow!("{e}"))?;
            reactor.drain_wakers(events)
        };
        
        for waker in wakers {
            waker.wake();
        }
        
        Ok(())
    }
}