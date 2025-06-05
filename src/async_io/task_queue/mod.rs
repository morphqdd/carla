use std::pin::Pin;

pub type LocalBoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct TaskQueue {
    tasks: Vec<Task>
}

pub struct Task {
    pub(crate) future: LocalBoxedFuture<'static, ()>
}

impl TaskQueue {
    pub fn new() -> Self {
        Self { tasks: vec![] }
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