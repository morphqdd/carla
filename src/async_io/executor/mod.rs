use crate::async_io::task_queue::{Task, TaskQueue};
use crate::async_io::waker_util::waker;

pub struct Executor {
    task_queue: TaskQueue
}

impl Executor {
    pub fn new() -> Self {
        Self { task_queue: TaskQueue::new() }
    }

    pub fn spawn(&mut self, f: impl Future<Output = ()> + Send + 'static) {
        self.task_queue.push(Task { future: Box::pin(f) });
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop() {
            let waker = waker(move || {

            });

            match task.future.as_mut().poll() {

            }
        }
    }
}