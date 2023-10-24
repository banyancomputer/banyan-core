#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QueueConfig {
    name: &'static str,
    worker_count: usize,
}

impl QueueConfig {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            worker_count: 1,
        }
    }

    pub fn with_worker_count(mut self, worker_count: usize) -> Self {
        self.worker_count = worker_count;
        self
    }

    pub fn worker_count(&self) -> usize {
        self.worker_count
    }
}
