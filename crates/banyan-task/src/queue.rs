use crate::TaskStore;

/// A named queue that's queryable over a task store.
pub struct Queue(&'static str);

#[derive(Debug, Serialize)]
pub struct TaskStoreMetrics {
    pub(crate) total: i32,
    pub(crate) new: i32,
    pub(crate) in_progress: i32,
    pub(crate) panicked: i32,
    pub(crate) retried: i32,
    pub(crate) cancelled: i32,
    pub(crate) errored: i32,
    pub(crate) completed: i32,
    pub(crate) timed_out: i32,
    pub(crate) dead: i32,
    pub(crate) scheduled: i32,
    pub(crate) scheduled_future: i32,
}

impl Queue {
    pub fn name(name: &'static str) -> Queue {
        Queue(name)
    }

    pub fn metrics<S>(&self, task_store: &S) -> QueueMetrics
    where
        S: TaskStore,
    {
        QueueMetrics {
            name: self.0,
            task_store,
        }
    }
}