use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::sync::mpsc::{Receiver, Sender};

pub type LocalBoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'a>>;

#[derive(Debug)]
pub struct TaskQueue {
    sender: Sender<Arc<Task>>,
    receiver: Mutex<Receiver<Arc<Task>>>,
    tasks: Vec<Arc<Task>>
}

pub struct Task {
    pub(crate) future: RwLock<LocalBoxedFuture<'static, anyhow::Result<()>>>
}

impl Debug for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Task {{ ... }}")
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueue {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver: Mutex::new(receiver), tasks: vec![] }
    }

    pub fn sender(&self) -> Sender<Arc<Task>> {
        self.sender.clone()
    }

    pub fn recv(&mut self) {
        for runnable in self.receiver.lock().unwrap().try_iter() {
            self.tasks.push(runnable);
        }
    }

    pub fn push(&mut self, task: Arc<Task>) {
        self.tasks.push(task);
    }

    pub fn pop(&mut self) -> Option<Arc<Task>> {
        self.tasks.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}