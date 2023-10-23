#![allow(dead_code)]

use chrono::NaiveDateTime;

use crate::Task;

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
            Some(sa) => sa,
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

pub mod tests {
    use super::{CurrentTask, NaiveDateTime};

    pub fn default_current_task() -> CurrentTask {
        CurrentTask {
            id: uuid::Uuid::new_v4().to_string(),
            current_attempt: 0,
            scheduled_at: NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            started_at: NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        }
    }

    pub fn increment_current_task_attempt_count(ct: &mut CurrentTask) -> &mut CurrentTask {
        ct.current_attempt += 1;
        ct
    }
}
