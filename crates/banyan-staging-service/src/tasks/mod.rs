use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::async_trait;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use uuid::Uuid;

const TASK_EXECUTION_TIMEOUT: Duration = Duration::from_secs(30);

#[async_trait]
pub trait TaskLike: Serialize + DeserializeOwned + Sync + Send {
    const MAX_RETRIES: usize = 3;
    const QUEUE_NAME: &'static str = "default";
    const TASK_NAME: &'static str;

    type Error: std::error::Error;
    type TaskContext: Clone + Send;

    async fn run(&self, ctx: Self::TaskContext) -> Result<(), Self::Error>;

    async fn unique_key(&self) -> Option<String> {
        None
    }
}

#[async_trait]
pub trait TaskLikeExt {
    async fn enqueue<S: TaskStore>(
        self,
        connection: &mut S::Connection,
    ) -> Result<Option<Uuid>, TaskQueueError>;
}

#[derive(Debug)]
pub enum TaskQueueError {
    UnknownTask(Uuid),
    Unknown,
}

impl Display for TaskQueueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use TaskQueueError::*;

        match &self {
            UnknownTask(id) => f.write_fmt(format_args!("unable to find task with ID {id}")),
            Unknown => f.write_str("unspecified error with the task queue"),
        }
    }
}

impl std::error::Error for TaskQueueError {}

#[async_trait]
pub trait TaskStore: Send + Sync + 'static {
    type Connection: Send;

    async fn cancel(&self, id: Uuid) -> Result<(), TaskQueueError> {
        self.update_state(id, TaskState::Cancelled).await
    }

    async fn enqueue<T: TaskLike>(
        conn: &mut Self::Connection,
        task: T,
    ) -> Result<Option<Uuid>, TaskQueueError>
    where
        Self: Sized;

    async fn next(&self, queue_name: &str) -> Result<Option<Task>, TaskQueueError>;

    async fn enqueue_retry(&self, id: Uuid) -> Result<Option<Uuid>, TaskQueueError>;

    async fn update_state(&self, id: Uuid, state: TaskState) -> Result<(), TaskQueueError>;
}

#[derive(Deserialize, Serialize)]
pub struct TestTask {
    number: usize,
}

impl TestTask {
    pub fn new(number: usize) -> Self {
        Self { number }
    }
}

#[async_trait]
impl TaskLike for TestTask {
    const TASK_NAME: &'static str = "test_task";

    type Error = TestTaskError;
    type TaskContext = ();

    async fn run(&self, _ctx: Self::TaskContext) -> Result<(), Self::Error> {
        tracing::info!("the test task value is {}", self.number);
        Ok(())
    }
}

#[derive(Debug)]
pub struct TestTaskError;

impl Display for TestTaskError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("unspecified error with the task")
    }
}

impl std::error::Error for TestTaskError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskState {
    New,
    InProgress,
    Complete,
    Error,
    TimedOut,
    Cancelled,
    Dead,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub id: Uuid,

    next_id: Option<Uuid>,
    previous_id: Option<Uuid>,

    name: String,
    queue_name: String,

    unique_key: Option<String>,
    state: TaskState,
    remaining_retries: usize,

    // will need a live-cancel signal and likely a custom Future impl to ensure its used for proper
    // timeout handling

    payload: serde_json::Value,
    error: Option<serde_json::Value>,

    scheduled_at: Instant,
    scheduled_to_run_at: Instant,

    started_at: Option<Instant>,
    finished_at: Option<Instant>,
}

#[derive(Clone, Default)]
pub struct MemoryTaskStore {
    pub tasks: Arc<Mutex<BTreeMap<Uuid, Task>>>,
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
            if !matches!(task.state, TaskState::New | TaskState::InProgress { .. }) {
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
    ) -> Result<Option<Uuid>, TaskQueueError> {
        let unique_key = task.unique_key().await;

        if let Some(new_key) = &unique_key {
            if MemoryTaskStore::is_key_present(&conn, &new_key).await {
                return Ok(None);
            }
        }

        let id = Uuid::new_v4();
        let payload = serde_json::to_value(task)
            .map_err(|_| TaskQueueError::Unknown)?;

        let task = Task {
            id: id.clone(),

            next_id: None,
            previous_id: None,

            name: T::TASK_NAME.to_string(),
            queue_name: T::QUEUE_NAME.to_string(),

            unique_key,
            state: TaskState::New,
            remaining_retries: T::MAX_RETRIES,

            payload,
            error: None,

            scheduled_at: Instant::now(),
            scheduled_to_run_at: Instant::now(),

            started_at: None,
            finished_at: None,
        };

        let mut tasks = conn.tasks.lock().await;
        tasks.insert(task.id, task);

        Ok(Some(id))
    }

    async fn enqueue_retry(&self, id: Uuid) -> Result<Option<Uuid>, TaskQueueError> {
        let mut tasks = self.tasks.lock().await;

        let target_task = match tasks.get_mut(&id) {
            Some(t) => t,
            None => {
                return Err(TaskQueueError::UnknownTask(id))
            },
        };

        // these states are the only retryable states
        if !matches!(target_task.state, TaskState::Error | TaskState::TimedOut) {
            tracing::warn!(?id, "task is not in a state that can be retried");
            return Err(TaskQueueError::Unknown);
        }

        // no retries remaining mark the task as dead
        if target_task.remaining_retries <= 0 {
            tracing::warn!(?id, "task failed with no more retries remaining");
            target_task.state = TaskState::Dead;
            return Ok(None);
        }

        let mut new_task = target_task.clone();

        let new_id = Uuid::new_v4();
        target_task.next_id = Some(new_task.id);

        new_task.id = new_id;
        new_task.previous_id = Some(target_task.id);

        new_task.remaining_retries -= 1;
        new_task.state = TaskState::New;
        new_task.scheduled_at = Instant::now();
        // for now just retry again in five minutes, will probably want some kind of backoff for
        // this
        new_task.scheduled_to_run_at = Instant::now() + Duration::from_secs(300);

        tasks.insert(new_task.id, new_task);

        tracing::info!(?id, ?new_id, "task will be retried in the future");

        Ok(Some(new_id))
    }

    async fn next(&self, queue_name: &str) -> Result<Option<Task>, TaskQueueError> {
        let mut tasks = self.tasks.lock().await;
        let mut next_task = None;

        let reference_time = Instant::now();
        let mut tasks_to_retry = Vec::new();

        for (id, task) in tasks
            .iter_mut()
            .filter(|(_, task)| task.scheduled_to_run_at <= reference_time)
            .sorted_by(|a, b| sort_tasks(a.1, b.1))
        {
            match (task.state, task.started_at) {
                (TaskState::New, None) => {
                    if task.queue_name != queue_name {
                        continue;
                    }

                    task.started_at = Some(Instant::now());
                    task.state = TaskState::InProgress;

                    next_task = Some(task.clone());
                    break;
                },
                (TaskState::InProgress, Some(started_at)) => {
                    if Instant::now().duration_since(started_at) >= TASK_EXECUTION_TIMEOUT {
                        // todo: need to send cancel signal to the task
                        task.state = TaskState::TimedOut;
                        task.finished_at = Some(Instant::now());

                        tasks_to_retry.push(id);

                        continue;
                    }
                },
                // cancelled is the only other state allowed to not have a started_at
                (TaskState::Cancelled, None) => (),
                (state, None) => {
                    tracing::error!(id = ?task.id, ?state, "encountered task in illegal state");

                    task.remaining_retries = 0;
                    task.state = TaskState::Error;
                    task.finished_at = Some(Instant::now());
                }
                _ => (),
            }
        }

        for id in tasks_to_retry.into_iter() {
            // this is best effort mostly for timed out
            let _ = self.enqueue_retry(*id).await;
        }

        Ok(next_task)
    }

    async fn update_state(&self, id: Uuid, new_state: TaskState) -> Result<(), TaskQueueError> {
        let mut tasks = self.tasks.lock().await;

        let task = match tasks.get_mut(&id) {
            Some(t) => t,
            None => {
                return Err(TaskQueueError::UnknownTask(id))
            },
        };

        if task.state != TaskState::InProgress {
            tracing::error!("only in progress tasks are allowed to transition to other states");
            return Err(TaskQueueError::Unknown);
        }

        match new_state {
            // this state should only exist when the task is first created
            TaskState::New => {
                tracing::error!("can't transition an existing task to the New state");
                return Err(TaskQueueError::Unknown);
            }
            // this is an internal transition that happens automatically when the task is picked up
            TaskState::InProgress => {
                tracing::error!("only the task store may transition a task to the InProgress state");
                return Err(TaskQueueError::Unknown);
            }
            _ => ()
        }

        task.finished_at = Some(Instant::now());
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
