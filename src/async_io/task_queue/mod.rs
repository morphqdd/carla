use std::pin::Pin;
use std::sync::{mpsc, Arc, RwLock};
use std::sync::mpsc::{Receiver, Sender};

pub type LocalBoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'a>>;

pub struct TaskQueue {
    sender: Sender<Task>,
    receiver: Receiver<Task>,
    tasks: Vec<Task>
}

#[derive(Clone)]
pub struct Task {
    pub(crate) future: Arc<RwLock<LocalBoxedFuture<'static, ()>>>
}

impl TaskQueue {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver, tasks: vec![] }
    }

    pub fn sender(&self) -> Sender<Task> {
        self.sender.clone()
    }

    pub fn recv(&mut self) {
        for runnable in self.receiver.try_iter() {
            self.tasks.push(runnable);
        }
    }

    pub fn push(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn pop(&mut self) -> Option<Task> {
        self.tasks.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}