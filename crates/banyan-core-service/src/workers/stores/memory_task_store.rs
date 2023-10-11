use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use itertools::Itertools;
use tokio::sync::Mutex;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::workers::{TASK_EXECUTION_TIMEOUT, Task, TaskId, TaskLike, TaskState, TaskStore, TaskStoreError};

#[derive(Clone, Default)]
pub struct MemoryTaskStore {
    pub tasks: Arc<Mutex<BTreeMap<TaskId, Task>>>,
}

impl MemoryTaskStore {
    // note: might want to extend this to be unique over a queue... I could just prepend the queue
    // the key or something...
    async fn is_key_present(conn: &Self, key: &str) -> bool {
        let tasks = conn.tasks.lock().await;

        for (_, task) in tasks.iter() {
            // we only need to look at a task if it also has a unique key
            let existing_key = match &task.unique_key {
                Some(ek) => ek,
                None => continue,
            };

            // any task that has already ended isn't considered for uniqueness checks
            if !matches!(task.state, TaskState::New | TaskState::InProgress | TaskState::Retry) {
                continue;
            }

            // we found a match, we don't need to enqueue a new task
            if key == existing_key {
                return true;
            }
        }

        false
    }
}

#[async_trait]
impl TaskStore for MemoryTaskStore {
    type Connection = Self;

    async fn enqueue<T: TaskLike>(
        conn: &mut Self::Connection,
        task: T,
    ) -> Result<Option<TaskId>, TaskStoreError> {
        let unique_key = task.unique_key().await;

        if let Some(ukey) = &unique_key {
            if MemoryTaskStore::is_key_present(conn, ukey).await {
                return Ok(None);
            }
        }

        let id = TaskId::from(Uuid::new_v4());
        let payload = serde_json::to_value(task).map_err(TaskStoreError::EncodeFailed)?;

        let task = Task {
            id,

            next_id: None,
            previous_id: None,

            name: T::TASK_NAME.to_string(),
            queue_name: T::QUEUE_NAME.to_string(),

            unique_key,
            state: TaskState::New,
            current_attempt: 0,
            maximum_attempts: T::MAX_RETRIES,

            payload,
            error: None,

            scheduled_at: OffsetDateTime::now_utc(),
            scheduled_to_run_at: OffsetDateTime::now_utc(),

            started_at: None,
            finished_at: None,
        };

        let mut tasks = conn.tasks.lock().await;
        tasks.insert(task.id, task);

        Ok(Some(id))
    }

    async fn next(&self, queue_name: &str, task_names: &[&str]) -> Result<Option<Task>, TaskStoreError> {
        let mut tasks = self.tasks.lock().await;
        let mut next_task = None;

        let reference_time = OffsetDateTime::now_utc();
        let mut tasks_to_retry = Vec::new();

        for (id, task) in tasks
            .iter_mut()
            .filter(|(_, task)| task_names.contains(&task.name.as_str()) && task.scheduled_to_run_at <= reference_time)
            // only care about tasks that have a state to advance
            .filter(|(_, task)| matches!(task.state, TaskState::New | TaskState::InProgress | TaskState::Retry))
            .sorted_by(|a, b| sort_tasks(a.1, b.1))
        {
            match (task.state, task.started_at) {
                (TaskState::New | TaskState::Retry, None) => {
                    if task.queue_name != queue_name {
                        continue;
                    }

                    task.started_at = Some(OffsetDateTime::now_utc());
                    task.state = TaskState::InProgress;

                    next_task = Some(task.clone());
                    break;
                }
                (TaskState::InProgress, Some(started_at)) => {
                    if (started_at + TASK_EXECUTION_TIMEOUT) >= OffsetDateTime::now_utc() {
                        // todo: need to send cancel signal to the task
                        task.state = TaskState::TimedOut;
                        task.finished_at = Some(OffsetDateTime::now_utc());

                        tasks_to_retry.push(id);
                    }
                }
                (state, _) => {
                    tracing::error!(id = ?task.id, ?state, "encountered task in illegal state");
                    task.state = TaskState::Dead;
                    task.finished_at = Some(OffsetDateTime::now_utc());
                }
            }
        }

        for id in tasks_to_retry.into_iter() {
            // attempt to requeue any of these tasks we encountered, if we fail to requeue them its
            // not a big deal but we will keep trying if they stay in that state... Might want to
            // put some kind of time window on these or something
            let _ = self.retry(*id).await;
        }

        Ok(next_task)
    }

    async fn retry(&self, id: TaskId) -> Result<Option<TaskId>, TaskStoreError> {
        let mut tasks = self.tasks.lock().await;

        let target_task = match tasks.get_mut(&id) {
            Some(t) => t,
            None => return Err(TaskStoreError::UnknownTask(id)),
        };

        // these states are the only retryable states
        if !matches!(target_task.state, TaskState::Error | TaskState::TimedOut) {
            tracing::warn!(?id, "task is not in a state that can be retried");
            return Err(TaskStoreError::NotRetryable(target_task.state));
        }

        // no retries remaining mark the task as dead
        if target_task.current_attempt >= target_task.maximum_attempts {
            tracing::warn!(?id, "task failed with no more attempts remaining");
            target_task.state = TaskState::Dead;
            return Ok(None);
        }

        let mut new_task = target_task.clone();

        let new_id = TaskId::from(Uuid::new_v4());
        target_task.next_id = Some(new_task.id);

        new_task.id = new_id;
        new_task.previous_id = Some(target_task.id);

        new_task.current_attempt += 1;
        new_task.state = TaskState::Retry;
        new_task.started_at = None;
        new_task.scheduled_at = OffsetDateTime::now_utc();

        // really rough exponential backoff, 4, 8, and 16 seconds by default probably want this to
        // be much longer...
        let backoff_secs = 2u64.saturating_pow(new_task.current_attempt.saturating_add(1) as u32);
        tracing::info!(?id, ?new_id, "task will be retried {backoff_secs} secs in the future");
        new_task.scheduled_to_run_at = OffsetDateTime::now_utc() + Duration::from_secs(backoff_secs);

        tasks.insert(new_task.id, new_task);

        Ok(Some(new_id))
    }

    async fn update_state(&self, id: TaskId, new_state: TaskState) -> Result<(), TaskStoreError> {
        let mut tasks = self.tasks.lock().await;

        let task = match tasks.get_mut(&id) {
            Some(t) => t,
            None => return Err(TaskStoreError::UnknownTask(id)),
        };

        if task.state != TaskState::InProgress {
            tracing::error!("only in progress tasks are allowed to transition to other states");
            return Err(TaskStoreError::InvalidStateTransition(task.state, new_state));
        }

        match new_state {
            // this state should only exist when the task is first created
            TaskState::New => {
                tracing::error!("can't transition an existing task to the New state");
                return Err(TaskStoreError::InvalidStateTransition(task.state, new_state));
            }
            // this is an internal transition that happens automatically when the task is picked up
            TaskState::InProgress => {
                tracing::error!(
                    "only the task store may transition a task to the InProgress state"
                );
                return Err(TaskStoreError::InvalidStateTransition(task.state, new_state));
            }
            _ => (),
        }

        task.finished_at = Some(OffsetDateTime::now_utc());
        task.state = new_state;

        Ok(())
    }
}

fn sort_tasks(a: &Task, b: &Task) -> Ordering {
    match a.scheduled_to_run_at.cmp(&b.scheduled_to_run_at) {
        Ordering::Equal => a.scheduled_at.cmp(&b.scheduled_at),
        ord => ord,
    }
}
