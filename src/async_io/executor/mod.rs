use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};
use crate::async_io::task_queue::{Task, TaskQueue};
use crate::async_io::waker_util::waker;
use anyhow::{anyhow, Result};
use polling::Events;
use crate::async_io::reactor::REACTOR;

pub struct Executor {
    task_queue: TaskQueue
}

impl Executor {
    pub fn new() -> Self {
        Self { task_queue: TaskQueue::new() }
    }

    pub fn spawn(&mut self, f: impl Future<Output = ()> + Send + Sync + 'static) {
        self.task_queue.push(Task { future: Arc::new(RwLock::new(Box::pin(f))) });
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            while let Some(task) = self.task_queue.pop() {
                let sender = self.task_queue.sender();
                let waker_task = task.clone();
                let waker = waker(move || {
                    sender.send(waker_task.clone()).unwrap();
                });

                let mut ctx = Context::from_waker(&waker);

                match task.future.write()
                    .map_err(|e| anyhow!("{e}"))?
                    .as_mut().poll(&mut ctx) {
                    Poll::Ready(_) => {}
                    Poll::Pending => {}
                }
            }

            self.wait_for_io()?;

            self.task_queue.recv();
        }
    }

    fn wait_for_io(&mut self) -> Result<()> {
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