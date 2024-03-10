use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::Arc;

use futures::future::join_all;
use futures::Future;
use time::OffsetDateTime;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use crate::task_like::RecurringTask;
use crate::{
    CurrentTask, QueueConfig, TaskExecError, TaskLike, TaskStore, TaskStoreError, Worker,
    WORKER_SHUTDOWN_TIMEOUT,
};

pub type ExecuteTaskFn<Context> = Arc<
    dyn Fn(
            CurrentTask,
            Vec<u8>,
            Context,
        ) -> Pin<Box<dyn Future<Output = Result<(), TaskExecError>> + Send>>
        + Send
        + Sync,
>;

pub type EnqueueRecurringTaskFn<S> = Arc<
    dyn Fn(
            <S as TaskStore>::Pool,
        ) -> Pin<Box<dyn Future<Output = Result<(), TaskExecError>> + Send>>
        + Send
        + Sync,
>;

pub type NextScheduleFn =
    Arc<dyn Fn(Vec<u8>) -> Result<Option<OffsetDateTime>, TaskExecError> + Send + Sync>;

pub type StateFn<Context> = Arc<dyn Fn() -> Context + Send + Sync>;

#[derive(Clone)]
pub struct WorkerPool<Context, S>
where
    Context: Clone + Send + 'static,
    S: TaskStore + Clone,
{
    context_data_fn: StateFn<Context>,
    pool_fn: StateFn<S::Pool>,
    task_store: S,
    task_registry: BTreeMap<&'static str, ExecuteTaskFn<Context>>,
    schedule_registry: BTreeMap<&'static str, NextScheduleFn>,
    startup_registry: BTreeMap<&'static str, EnqueueRecurringTaskFn<S>>,
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

    pub fn new<A, B>(task_store: S, pool_fn: A, context_data_fn: B) -> Self
    where
        A: Fn() -> S::Pool + Send + Sync + 'static,
        B: Fn() -> Context + Send + Sync + 'static,
    {
        Self {
            context_data_fn: Arc::new(context_data_fn),
            pool_fn: Arc::new(pool_fn),
            task_store,
            task_registry: BTreeMap::new(),
            schedule_registry: BTreeMap::new(),
            startup_registry: BTreeMap::new(),
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
            .or_default()
            .push(TL::TASK_NAME);

        self.task_registry
            .insert(TL::TASK_NAME, Arc::new(deserialize_and_run_task::<TL>));

        self
    }

    pub async fn register_recurring_task_type<RT>(mut self) -> Result<Self, TaskStoreError>
    where
        RT: RecurringTask<Context = Context>,
    {
        self.schedule_registry
            .insert(RT::TASK_NAME, Arc::new(next_schedule::<RT>));

        self.startup_registry
            .insert(RT::TASK_NAME, Arc::new(enqueue_recurring_task::<RT, S>));

        Ok(self.register_task_type::<RT>())
    }

    pub async fn start<F>(self, shutdown_signal: F) -> Result<JoinHandle<()>, WorkerPoolError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        for (_task_name, enqueue_recurring_task_fn) in self.startup_registry.iter() {
            let pool = (self.pool_fn)();
            let queue_result = enqueue_recurring_task_fn(pool).await;
            tracing::info!("result: {:?}", queue_result);
        }

        for (queue_name, queue_tracked_tasks) in self.queue_tasks.iter() {
            if !self.worker_queues.contains_key(queue_name) {
                return Err(WorkerPoolError::QueueNotConfigured(
                    queue_name,
                    queue_tracked_tasks.clone(),
                ));
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
                    self.schedule_registry.clone(),
                    Some(inner_shutdown_rx.clone()),
                );

                let worker_handle = tokio::spawn(async move {
                    match worker.run_tasks().await {
                        Ok(()) => {
                            tracing::info!(name = ?worker_name, "worker stopped successfully")
                        }
                        Err(err) => {
                            tracing::error!(name = ?worker_name, "worker stopped due to error: {err}")
                        }
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
            let worker_errors: Vec<_> = match timeout(
                WORKER_SHUTDOWN_TIMEOUT,
                join_all(worker_handles),
            )
            .await
            {
                Ok(res) => res
                    .into_iter()
                    .filter(Result::is_err)
                    .map(Result::unwrap_err)
                    .collect(),
                Err(_) => {
                    tracing::warn!("timed out waiting for workers to shutdown, not reporting outstanding errors");
                    Vec::new()
                }
            };

            if worker_errors.is_empty() {
                tracing::info!("worker pool shutdown gracefully");
            } else {
                tracing::error!(
                    "workers reported the following errors during shutdown:\n{:?}",
                    worker_errors
                );
            }
        });

        Ok(shutdown_guard)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerPoolError {
    #[error("found named queue '{0}' defined by task(s) {1:?} that doesn't have a matching queue config")]
    QueueNotConfigured(&'static str, Vec<&'static str>),

    #[error("failed to add the recurring task {0}, to the startup queue: {1}")]
    FailedToQueueRecurring(String, TaskStoreError),
}

fn deserialize_and_run_task<TL>(
    current_task: CurrentTask,
    payload: Vec<u8>,
    context: TL::Context,
) -> Pin<Box<dyn Future<Output = Result<(), TaskExecError>> + Send>>
where
    TL: TaskLike,
{
    Box::pin(async move {
        serde_json::from_slice::<TL>(&payload)?
            .run(current_task, context)
            .await
            .map(|_| ())
            .map_err(|err| TaskExecError::ExecutionFailed(err.to_string()))
    })
}

fn next_schedule<RT: RecurringTask>(
    payload: Vec<u8>,
) -> Result<Option<OffsetDateTime>, TaskExecError> {
    let task: RT = serde_json::from_slice(&payload)?;
    Ok(task.next_schedule())
}

fn enqueue_recurring_task<RT, S>(
    mut pool: S::Pool,
) -> Pin<Box<dyn Future<Output = Result<(), TaskExecError>> + Send>>
where
    RT: RecurringTask,
    S: TaskStore,
{
    Box::pin(async move {
        S::enqueue(&mut pool, RT::default())
            .await
            .map(|_| ())
            .map_err(|err| TaskExecError::ExecutionFailed(err.to_string()))
    })
}
