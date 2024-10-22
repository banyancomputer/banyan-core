use time::OffsetDateTime;

use crate::Task;

pub struct CurrentTask {
    id: String,
    current_attempt: i64,
    scheduled_at: OffsetDateTime,
    started_at: OffsetDateTime,
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

impl Default for CurrentTask {
    fn default() -> CurrentTask {
        CurrentTask {
            id: uuid::Uuid::new_v4().to_string(),
            current_attempt: 0,
            scheduled_at: OffsetDateTime::UNIX_EPOCH,
            started_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CurrentTaskError {
    #[error("task must be started before creating a current instance")]
    TaskNotStarted,
}

#[cfg(any(test, feature = "test-utils"))]
pub mod tests {
    use super::CurrentTask;

    pub fn increment_current_task_attempt_count(ct: &mut CurrentTask) -> &mut CurrentTask {
        ct.current_attempt += 1;
        ct
    }
}
