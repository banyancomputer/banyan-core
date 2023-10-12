use chrono::NaiveDateTime;

use crate::workers::Task;

pub struct CurrentTask {
    id: String,
    current_attempt: i64,
    scheduled_at: NaiveDateTime,
    started_at: NaiveDateTime,
}

impl CurrentTask {
    pub fn current_attempt(&self) -> i64 {
        self.current_attempt
    }

    pub fn new(task: &Task) -> Self {
        Self {
            id: task.id.clone(),
            current_attempt: task.current_attempt,
            scheduled_at: task.scheduled_at,
            started_at: task.started_at.expect("task to be started"),
        }
    }
}
