use time::OffsetDateTime;

use crate::workers::{Task, TaskId};

pub struct CurrentTask {
    id: TaskId,
    current_attempt: usize,
    scheduled_at: OffsetDateTime,
    started_at: OffsetDateTime,
}

impl CurrentTask {
    pub fn current_attempt(&self) -> usize {
        self.current_attempt
    }

    pub fn new(task: &Task) -> Self {
        Self {
            id: task.id,
            current_attempt: task.current_attempt,
            scheduled_at: task.scheduled_at,
            started_at: task.started_at.expect("task to be started"),
        }
    }
}
