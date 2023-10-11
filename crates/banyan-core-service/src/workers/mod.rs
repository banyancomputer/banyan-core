use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use axum::async_trait;
use futures::{Future, FutureExt};
use futures::future::{BoxFuture, join_all};
use itertools::Itertools;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::{Mutex, watch};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use uuid::Uuid;

// constants

const MAXIMUM_CHECK_DELAY: Duration = Duration::from_millis(250);

const TASK_EXECUTION_TIMEOUT: Duration = Duration::from_secs(30);

const WORKER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

// types

pub type ExecuteTaskFn<Context> = Arc<
    dyn Fn(
        CurrentTask,
        serde_json::Value,
        Context,
    ) -> Pin<Box<dyn Future<Output = Result<(), TaskExecError>> + Send>>
    + Send
    + Sync,
>;

pub type StateFn<Context> = Arc<dyn Fn() -> Context + Send + Sync>;

// traits

#[async_trait]
pub trait TaskLike: Serialize + DeserializeOwned + Sync + Send + 'static {
    // todo: rename MAX_ATTEMPTS
    const MAX_RETRIES: usize = 3;

    const QUEUE_NAME: &'static str = "default";

    const TASK_NAME: &'static str;

    type Error: std::error::Error;
    type Context: Clone + Send + 'static;

    async fn run(&self, task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error>;

    async fn unique_key(&self) -> Option<String> {
        None
    }
}

#[async_trait]
pub trait TaskLikeExt {
    async fn enqueue<S: TaskStore>(
        self,
        connection: &mut S::Connection,
    ) -> Result<Option<TaskId>, TaskQueueError>;
}

#[async_trait]
impl<T> TaskLikeExt for T
where
    T: TaskLike,
{
    async fn enqueue<S: TaskStore>(
        self,
        connection: &mut S::Connection,
    ) -> Result<Option<TaskId>, TaskQueueError> {
        S::enqueue(connection, self).await
    }
}

#[async_trait]
pub trait TaskStore: Send + Sync + 'static {
    type Connection: Send;

    async fn cancel(&self, id: TaskId) -> Result<(), TaskQueueError> {
        self.update_state(id, TaskState::Cancelled).await
    }

    async fn completed(&self, id: TaskId) -> Result<(), TaskQueueError> {
        self.update_state(id, TaskState::Complete).await
    }

    async fn enqueue<T: TaskLike>(
        conn: &mut Self::Connection,
        task: T,
    ) -> Result<Option<TaskId>, TaskQueueError>
    where
        Self: Sized;

    async fn errored(&self, id: TaskId, error: TaskExecError) -> Result<Option<TaskId>, TaskQueueError> {
        use TaskExecError as TEE;

        match error {
            TEE::DeserializationFailed(_) | TEE::Panicked(_) => {
                self.update_state(id, TaskState::Dead).await?;
                Ok(None)
            }
            TEE::ExecutionFailed(_) => {
                self.update_state(id, TaskState::Error).await?;
                self.retry(id).await
            }
        }
    }

    async fn next(&self, queue_name: &str, task_names: &[&str]) -> Result<Option<Task>, TaskQueueError>;

    async fn retry(&self, id: TaskId) -> Result<Option<TaskId>, TaskQueueError>;

    async fn update_state(&self, id: TaskId, state: TaskState) -> Result<(), TaskQueueError>;
}

// structs

struct CatchPanicFuture<F: Future + Send + 'static> {
    inner: BoxFuture<'static, F::Output>,
}

impl<F: Future + Send + 'static> CatchPanicFuture<F> {
    pub fn wrap(f: F) -> Self {
        Self { inner: f.boxed() }
    }
}

impl<F: Future + Send + 'static> Future for CatchPanicFuture<F> {
    type Output = Result<F::Output, CaughtPanic>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = &mut self.inner;

        match catch_unwind(move || inner.poll_unpin(cx)) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(value)) => Poll::Ready(Ok(value)),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

#[derive(Debug)]
pub struct CaughtPanic(String);

impl Display for CaughtPanic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "panicked message: {}", self.0)
    }
}

impl std::error::Error for CaughtPanic {}

#[derive(Deserialize, Serialize)]
pub struct CreateTask {
    name: String,
    queue_name: String,

    payload: serde_json::Value,
    maximum_attempts: usize,

    scheduled_to_run_at: OffsetDateTime,
}

impl CreateTask {
    fn new<T: TaskLike>(task: T, run_at: OffsetDateTime) -> Self {
        Self {
            name: T::TASK_NAME.to_string(),
            queue_name: T::QUEUE_NAME.to_string(),

            payload: serde_json::to_value(&task).expect("valid encoding"),
            maximum_attempts: T::MAX_RETRIES,

            scheduled_to_run_at: run_at,
        }
    }
}

pub struct CurrentTask {
    id: TaskId,
    current_attempt: usize,
    scheduled_at: OffsetDateTime,
    started_at: OffsetDateTime,
}

impl CurrentTask {
    pub fn new(task: &Task) -> Self {
        Self {
            id: task.id,
            current_attempt: task.current_attempt,
            scheduled_at: task.scheduled_at,
            started_at: task.started_at.expect("task to be started"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QueueConfig {
    name: &'static str,
    worker_count: usize,
}

impl QueueConfig {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            worker_count: 1,
        }
    }

    pub fn worker_count(mut self, worker_count: usize) -> Self {
        self.worker_count = worker_count;
        self
    }
}

#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd, Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct TaskId(Uuid);

impl Debug for TaskId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TaskId").field(&self.0).finish()
    }
}

impl Display for TaskId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for TaskId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<TaskId> for Uuid {
    fn from(value: TaskId) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub id: TaskId,

    next_id: Option<TaskId>,
    previous_id: Option<TaskId>,

    name: String,
    queue_name: String,

    unique_key: Option<String>,
    state: TaskState,

    current_attempt: usize,
    maximum_attempts: usize,

    // will need a live-cancel signal and likely a custom Future impl to ensure its used for proper
    // timeout handling

    payload: serde_json::Value,
    error: Option<String>,

    scheduled_at: OffsetDateTime,
    scheduled_to_run_at: OffsetDateTime,

    started_at: Option<OffsetDateTime>,
    finished_at: Option<OffsetDateTime>,
}

#[derive(Debug, thiserror::Error)]
pub enum TaskExecError {
    #[error("task deserialization failed: {0}")]
    DeserializationFailed(#[from] serde_json::Error),

    #[error("task execution failed: {0}")]
    ExecutionFailed(String),

    #[error("task panicked: {0}")]
    Panicked(#[from] CaughtPanic),
}

#[derive(Debug, thiserror::Error)]
pub enum TaskQueueError {
    #[error("unable to find task with ID {0}")]
    UnknownTask(TaskId),

    #[error("I lazily hit one of the queue errors I haven't implemented yet")]
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskState {
    New,
    InProgress,
    Panicked,
    Retry,
    Cancelled,
    Error,
    Complete,
    TimedOut,
    Dead,
}

struct Worker<Context, S>
where
    Context: Clone + Send + 'static,
    S: TaskStore + Clone,
{
    name: String,
    queue_config: QueueConfig,

    context_data_fn: StateFn<Context>,
    store: S,
    task_registry: BTreeMap<&'static str, ExecuteTaskFn<Context>>,

    shutdown_signal: Option<tokio::sync::watch::Receiver<()>>,
}

impl<Context, S> Worker<Context, S>
where
    Context: Clone + Send + 'static,
    S: TaskStore + Clone,
{
    fn new(
        name: String,
        queue_config: QueueConfig,
        context_data_fn: StateFn<Context>,
        store: S,
        task_registry: BTreeMap<&'static str, ExecuteTaskFn<Context>>,
        shutdown_signal: Option<tokio::sync::watch::Receiver<()>>,
    ) -> Self {
        Self {
            name,
            queue_config,
            context_data_fn,
            store,
            task_registry,
            shutdown_signal,
        }
    }

    async fn run(&self, task: Task) -> Result<(), WorkerError> {
        let task_info = CurrentTask::new(&task);

        let deserialize_and_run_task_fn = self.task_registry
            .get(task.name.as_str())
            .ok_or_else(|| WorkerError::UnregisteredTaskName(task.name))?
            .clone();

        let safe_runner = CatchPanicFuture::wrap({
            let context = (self.context_data_fn)();
            let payload = task.payload.clone();

            async move { deserialize_and_run_task_fn(task_info, payload, context).await }
        });

        // an error here occurs only when the task panicks, deserialization and regular task
        // execution errors are handled next
        //
        // todo: should note the task as having panicked if that's why this failed. There is also a
        // chance that the worker is corrupted in some way by the panic so I should set a flag on
        // this worker and handle two consecutive panics as a worker problem. The second task
        // triggering the panic should be presumed innocent and restored to a runnable state.
        let task_result = match safe_runner.await {
            Ok(tr) => tr,
            Err(err) => {
                tracing::error!("task panicked: {err}");

                // todo: save panic message into the task.error and save it back to the memory
                // store somehow...
                self.store.update_state(task.id, TaskState::Panicked)
                    .await
                    .map_err(WorkerError::UpdateTaskStatusFailed)?;

                // we didn't complete successfully, but we do want to keep processing tasks for
                // now. We may be corrupted due to the panic somehow if additional errors crop up.
                // Left as future work to handle this edge case.
                return Ok(());
            }
        };

        match task_result {
            Ok(_) => {
                self.store.update_state(task.id, TaskState::Complete)
                    .await
                    .map_err(WorkerError::UpdateTaskStatusFailed)?;
            }
            Err(err) => {
                tracing::error!("task failed with error: {err}");

                self.store.errored(task.id, TaskExecError::ExecutionFailed(err.to_string()))
                    .await
                    .map_err(WorkerError::RetryTaskFailed)?;
            }
        }

        Ok(())
    }

    async fn run_tasks(&mut self) -> Result<(), WorkerError> {
        let relevant_task_names: Vec<&'static str> = self.task_registry.keys().cloned().collect();

        loop {
            // check to see if its time to shutdown the worker
            //
            // todo: turn this into a select with a short fallback timeout on task execution to try
            // and finish it within our graceful shutdown window
            if let Some(shutdown_signal) = &self.shutdown_signal {
                match shutdown_signal.has_changed() {
                    Ok(true) => return Ok(()),
                    Err(_) => return Err(WorkerError::EmergencyShutdown),
                    _ => (),
                }
            }

            let next_task = self.store
                .next(self.queue_config.name, &relevant_task_names)
                .await
                .map_err(WorkerError::StoreUnavailable)?;

            if let Some(task) = next_task {
                tracing::info!(id = ?task.id, "starting execution of task");
                self.run(task).await?;
                continue;
            }

            // todo this should probably be handled by some form of a centralized wake up manager
            // when things are enqueued which can also 'alarm' when a pending task is ready to be
            // scheduled instead of relying... and that change should probably be done using
            // future wakers instead of internal timeouts but some central scheduler
            match &mut self.shutdown_signal {
                Some(ss) => {
                    if let Ok(_signaled) = tokio::time::timeout(MAXIMUM_CHECK_DELAY, ss.changed()).await {
                        // todo might want to handle graceful / non-graceful differently
                        tracing::info!("received worker shutdown signal while idle");
                        return Ok(());
                    }

                    // intentionally letting the 'error' type fall through here as it means we
                    // timed out on waiting for a shutdown signal and should continue
                }
                None => {
                    tracing::info!("no tasks available for worker, sleeping for a time...");
                    let _ = tokio::time::sleep(MAXIMUM_CHECK_DELAY).await;
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("worker detected an error in the shutdown channel and forced and immediate exit")]
    EmergencyShutdown,

    #[error("failed to enqueue a failed task for re-execution: {0}")]
    RetryTaskFailed(TaskQueueError),

    #[error("error while attempting to retrieve the next task: {0}")]
    StoreUnavailable(TaskQueueError),

    #[error("failed to update task status with store: {0}")]
    UpdateTaskStatusFailed(TaskQueueError),

    #[error("during execution of a dequeued task, encountered unregistered task '{0}'")]
    UnregisteredTaskName(String),
}

#[derive(Clone)]
pub struct WorkerPool<Context, S>
where
    Context: Clone + Send + 'static,
    S: TaskStore + Clone,
{
    context_data_fn: StateFn<Context>,
    task_store: S,
    task_registry: BTreeMap<&'static str, ExecuteTaskFn<Context>>,

    queue_tasks: BTreeMap<&'static str, Vec<&'static str>>,
    worker_queues: BTreeMap<&'static str, QueueConfig>,
}

impl<Context, S> WorkerPool<Context, S>
where
    Context: Clone + Send + 'static,
    S: TaskStore + Clone,
{
    pub fn configure_queue(mut self, config: QueueConfig) -> Self {
        self.worker_queues.insert(config.name, config);
        self
    }

    pub fn new<A>(task_store: S, context_data_fn: A) -> Self
    where
        A: Fn() -> Context + Send + Sync + 'static,
    {
        Self {
            context_data_fn: Arc::new(context_data_fn),
            task_store,
            task_registry: BTreeMap::new(),

            queue_tasks: BTreeMap::new(),
            worker_queues: BTreeMap::new(),
        }
    }

    pub fn register_task_type<TL>(mut self) -> Self
    where
        TL: TaskLike<Context = Context>,
    {
        self.queue_tasks
            .entry(TL::QUEUE_NAME)
            .or_insert_with(Vec::new)
            .push(TL::TASK_NAME);

        self.task_registry
            .insert(TL::TASK_NAME, Arc::new(deserialize_and_run_task::<TL>));

        self
    }

    pub async fn start<F>(self, shutdown_signal: F) -> Result<JoinHandle<()>, WorkerPoolError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        for (queue_name, queue_tracked_tasks) in self.queue_tasks.iter() {
            if !self.worker_queues.contains_key(queue_name) {
                return Err(WorkerPoolError::QueueNotConfigured(queue_name, queue_tracked_tasks.clone()));
            }
        }

        let (inner_shutdown_tx, inner_shutdown_rx) = watch::channel(());
        let mut worker_handles = Vec::new();

        for (queue_name, queue_config) in self.worker_queues.iter() {
            for idx in 0..queue_config.worker_count {
                let worker_name = format!("worker-{queue_name}-{idx}");

                // todo: make the worker_name into a span attached to this future and drop it from
                // the worker attributes

                let mut worker: Worker<Context, S> = Worker::new(
                    worker_name.clone(),
                    queue_config.clone(),
                    self.context_data_fn.clone(),
                    self.task_store.clone(),
                    self.task_registry.clone(),
                    Some(inner_shutdown_rx.clone()),
                );

                let worker_handle = tokio::spawn(async move {
                    match worker.run_tasks().await {
                        Ok(()) => tracing::info!(name = ?worker_name, "worker stopped successfully"),
                        Err(err) => tracing::error!(name = ?worker_name, "worker stopped due to error: {err}"),
                    }
                });

                worker_handles.push(worker_handle);
            }
        }

        let shutdown_guard = tokio::spawn(async move {
            // Wait until we receive a shutdown signal directly or the channel errors out due to
            // the other side being dropped
            let _ = shutdown_signal.await;

            // In either case, its time to shut things down. Let's try and notify our workers for
            // graceful shutdown.
            let _ = inner_shutdown_tx.send(());

            // try and collect error from workers but if it takes too long abandon them
            let worker_errors: Vec<_> = match timeout(WORKER_SHUTDOWN_TIMEOUT, join_all(worker_handles)).await {
                Ok(res) => res.into_iter().filter(Result::is_err).map(Result::unwrap_err).collect(),
                Err(_) => {
                    tracing::warn!("timed out waiting for workers to shutdown, not reporting outstanding errors");
                    Vec::new()
                }
            };

            if worker_errors.is_empty() {
                tracing::info!("worker pool shutdown gracefully");
            } else {
                tracing::error!("workers reported the following errors during shutdown:\n{:?}", worker_errors);
            }
        });

        Ok(shutdown_guard)
    }
}

#[derive(Clone)]
pub struct WorkScheduler<T: TaskStore>(T);

impl<T: TaskStore> WorkScheduler<T> {
    pub fn new(store: T) -> Self {
        Self(store)
    }
}

impl<T: TaskStore> Deref for WorkScheduler<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: TaskStore> DerefMut for WorkScheduler<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkSchedulerError {
    #[error("failed to enqueue task to workers: {0}")]
    EnqueueFailed(TaskQueueError),
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerPoolError {
    #[error("found named queue '{0}' defined by task(s) {1:?} that doesn't have a matching queue config")]
    QueueNotConfigured(&'static str, Vec<&'static str>),
}

// local helper functions

fn catch_unwind<F: FnOnce() -> R, R>(f: F) -> Result<R, CaughtPanic> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(res) => Ok(res),
        Err(panic_err) => {
            if let Some(msg) = panic_err.downcast_ref::<&'static str>() {
                return Err(CaughtPanic(msg.to_string()));
            }

            if let Some(msg) = panic_err.downcast_ref::<String>() {
                return Err(CaughtPanic(msg.to_string()));
            }

            Err(CaughtPanic("unknown panic message format".to_string()))
        },
    }
}

fn deserialize_and_run_task<TL>(
    current_task: CurrentTask,
    payload: serde_json::Value,
    context: TL::Context,
) -> Pin<Box<dyn Future<Output = Result<(), TaskExecError>> + Send>>
where
    TL: TaskLike,
{
    Box::pin(async move {
        let task: TL = serde_json::from_value(payload)?;

        match task.run(current_task, context).await {
            Ok(_) => Ok(()),
            Err(err) => Err(TaskExecError::ExecutionFailed(err.to_string())),
        }
    })
}

fn sort_tasks(a: &Task, b: &Task) -> Ordering {
    match a.scheduled_to_run_at.cmp(&b.scheduled_to_run_at) {
        Ordering::Equal => a.scheduled_at.cmp(&b.scheduled_at),
        ord => ord,
    }
}

// concrete work store implementation

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
    ) -> Result<Option<TaskId>, TaskQueueError> {
        let unique_key = task.unique_key().await;

        if let Some(new_key) = &unique_key {
            if MemoryTaskStore::is_key_present(conn, new_key).await {
                return Ok(None);
            }
        }

        let id = TaskId::from(Uuid::new_v4());
        let payload = serde_json::to_value(task).map_err(|_| TaskQueueError::Unknown)?;

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

    async fn next(&self, queue_name: &str, task_names: &[&str]) -> Result<Option<Task>, TaskQueueError> {
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

    async fn retry(&self, id: TaskId) -> Result<Option<TaskId>, TaskQueueError> {
        let mut tasks = self.tasks.lock().await;

        let target_task = match tasks.get_mut(&id) {
            Some(t) => t,
            None => return Err(TaskQueueError::UnknownTask(id)),
        };

        // these states are the only retryable states
        if !matches!(target_task.state, TaskState::Error | TaskState::TimedOut) {
            tracing::warn!(?id, "task is not in a state that can be retried");
            return Err(TaskQueueError::Unknown);
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

        // really rough exponential backoff, 4, 8, and 16 seconds by default
        let backoff_secs = 2u64.saturating_pow(new_task.current_attempt.saturating_add(1) as u32);
        tracing::info!(?id, ?new_id, "task will be retried {backoff_secs} secs in the future");
        new_task.scheduled_to_run_at = OffsetDateTime::now_utc() + Duration::from_secs(backoff_secs);

        tasks.insert(new_task.id, new_task);

        Ok(Some(new_id))
    }

    async fn update_state(&self, id: TaskId, new_state: TaskState) -> Result<(), TaskQueueError> {
        let mut tasks = self.tasks.lock().await;

        let task = match tasks.get_mut(&id) {
            Some(t) => t,
            None => return Err(TaskQueueError::UnknownTask(id)),
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
                tracing::error!(
                    "only the task store may transition a task to the InProgress state"
                );
                return Err(TaskQueueError::Unknown);
            }
            _ => (),
        }

        task.finished_at = Some(OffsetDateTime::now_utc());
        task.state = new_state;

        Ok(())
    }
}

// sample context implementation

// todo

// concrete task implementation

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
    type Context = ();

    async fn run(&self, task: CurrentTask, _ctx: Self::Context) -> Result<(), Self::Error> {
        if task.current_attempt != 0 {
            tracing::info!("the test task value is {}", self.number);
            return Ok(());
        }

        Err(TestTaskError::Unknown)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TestTaskError {
    #[error("unspecified error with the task")]
    Unknown,
}
