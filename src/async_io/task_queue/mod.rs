use std::pin::Pin;
use std::sync::{mpsc, Arc, RwLock};
use std::sync::mpsc::{Receiver, Sender};

pub type LocalBoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'a>>;

pub struct TaskQueue<T: Send + 'static> {
    sender: Sender<Arc<Task<T>>>,
    receiver: Receiver<Arc<Task<T>>>,
    tasks: Vec<Arc<Task<T>>>
}

pub struct Task<T: Send + 'static> {
    pub(crate) future: RwLock<LocalBoxedFuture<'static, T>>
}

impl<T: Send + 'static> TaskQueue<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver, tasks: vec![] }
    }

    pub fn sender(&self) -> Sender<Arc<Task<T>>> {
        self.sender.clone()
    }

    pub fn recv(&mut self) {
        for runnable in self.receiver.try_iter() {
            self.tasks.push(runnable);
        }
    }

    pub fn push(&mut self, task: Arc<Task<T>>) {
        self.tasks.push(task);
    }

    pub fn pop(&mut self) -> Option<Arc<Task<T>>> {
        self.tasks.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}