#![allow(dead_code)]

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
}

impl TryFrom<&Task> for CurrentTask {
    type Error = CurrentTaskError;

    fn try_from(value: &Task) -> Result<Self, Self::Error> {
        let started_at = match value.started_at {
            Some(sa) => sa.clone(),
            None => return Err(CurrentTaskError::TaskNotStarted),
        };

        Ok(Self {
            id: value.id.clone(),
            current_attempt: value.current_attempt,
            scheduled_at: value.scheduled_at,
            started_at,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CurrentTaskError {
    #[error("task must be started before creating a current instance")]
    TaskNotStarted,
}
