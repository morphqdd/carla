use std::pin::Pin;
use std::sync::{mpsc, Arc, RwLock};
use std::sync::mpsc::{Receiver, Sender};

pub type LocalBoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct TaskQueue {
    sender: Sender<Arc<Task>>,
    receiver: Receiver<Arc<Task>>,
    tasks: Vec<Arc<Task>>
}

pub struct Task {
    pub(crate) future: RwLock<LocalBoxedFuture<'static, ()>>
}

impl TaskQueue {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver, tasks: vec![] }
    }

    pub fn sender(&self) -> Sender<Arc<Task>> {
        self.sender.clone()
    }

    pub fn recv(&mut self) {
        for runnable in self.receiver.try_iter() {
            self.tasks.push(runnable);
        }
    }

    pub fn push(&mut self, task: Task) {
        self.tasks.push(Arc::new(task));
    }

    pub fn pop(&mut self) -> Option<Arc<Task>> {
        self.tasks.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}