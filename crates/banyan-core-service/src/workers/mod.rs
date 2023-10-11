use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use axum::async_trait;
use futures::{Future, FutureExt};
use futures::future::join_all;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::{Mutex, watch};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use uuid::Uuid;

mod create_task;
mod current_task;
pub mod panic_safe_future;
mod queue_config;
mod stores;
mod task;
mod task_id;
mod task_like;
mod task_state;
mod task_store;
pub mod tasks;

pub use create_task::CreateTask;
pub use current_task::CurrentTask;
pub use queue_config::QueueConfig;
pub use task_id::TaskId;
pub use task_like::{TaskLike, TaskLikeExt};
pub use task_state::TaskState;
pub use task_store::{TaskStore, TaskStoreError};
pub use task::{Task, TaskExecError};

pub const MAXIMUM_CHECK_DELAY: Duration = Duration::from_millis(250);

pub const TASK_EXECUTION_TIMEOUT: Duration = Duration::from_secs(30);

pub const WORKER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

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

        let safe_runner = panic_safe_future::PanicSafeFuture::wrap({
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
                .next(self.queue_config.name(), &relevant_task_names)
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
    RetryTaskFailed(TaskStoreError),

    #[error("error while attempting to retrieve the next task: {0}")]
    StoreUnavailable(TaskStoreError),

    #[error("failed to update task status with store: {0}")]
    UpdateTaskStatusFailed(TaskStoreError),

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
        self.worker_queues.insert(config.name(), config);
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
            for idx in 0..queue_config.worker_count() {
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
    EnqueueFailed(TaskStoreError),
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerPoolError {
    #[error("found named queue '{0}' defined by task(s) {1:?} that doesn't have a matching queue config")]
    QueueNotConfigured(&'static str, Vec<&'static str>),
}

// local helper functions

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
