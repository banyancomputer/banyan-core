use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::Arc;

use futures::future::join_all;
use futures::Future;
use sqlx::{Database, Executor, Sqlite};
use std::marker::PhantomData;
use time::OffsetDateTime;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use crate::task_like::RecurringTask;
use crate::{
    CurrentTask, QueueConfig, TaskExecError, TaskLike, TaskStore, TaskStoreError, Worker,
    WORKER_SHUTDOWN_TIMEOUT,
};

//type exec = for<'e> &'e mut C: Executor<'e, Database = Sqlite>;

//ktype Exec = dyn Executor<'_, Database = Sqlite>;

pub type ExecuteTaskFn<Context> = Arc<
    dyn Fn(
            CurrentTask,
            Vec<u8>,
            Context,
        ) -> Pin<Box<dyn Future<Output = Result<(), TaskExecError>> + Send>>
        + Send
        + Sync,
>;

pub type EnqueueRecurringTaskFn<E, D>
where
    D: Database,
    for<'e> &'e mut E: Executor<'e, Database = D>,
    E: std::marker::Send + 'static,
= Arc<
    dyn Fn(
        PhantomData<D>,
    ) -> Pin<
        Box<
            dyn Fn(&mut E) -> Pin<Box<dyn Future<Output = Result<Option<String>, TaskStoreError>>>>,
        >,
    >,
>;

pub type NextScheduleFn =
    Arc<dyn Fn(Vec<u8>) -> Result<Option<OffsetDateTime>, TaskExecError> + Send + Sync>;

pub type StateFn<State> = Arc<dyn Fn() -> State + Send + Sync>;

/*
pub trait Contexxt<D, E>: Clone + Send + Sync + 'static
where
    for<'e> &'e mut E: Executor<'e, Database = D>,
{
    //
    //fn executor(&self) -> E;
}
*/

#[derive(Clone)]
pub struct WorkerPool<C, S, D>
where
    //C: Contexxt<D, E>,
    C: Clone + Send + Sync + 'static,
    D: Database + Sync,
    S: TaskStore<D> + Clone,
{
    _lifetime: PhantomData<D>,
    context_fn: StateFn<C>,
    task_store: S,
    task_registry: BTreeMap<&'static str, ExecuteTaskFn<C>>,
    schedule_registry: BTreeMap<&'static str, NextScheduleFn>,
    queue_tasks: BTreeMap<&'static str, Vec<&'static str>>,
    worker_queues: BTreeMap<&'static str, QueueConfig>,
}

impl<C, D, S> WorkerPool<C, S, D>
where
    //C: Contexxt<D, E>,
    C: Clone + Send + Sync + 'static,
    D: Database + Sync,
    S: TaskStore<D> + Clone,
{
    pub fn configure_queue(mut self, config: QueueConfig) -> Self {
        self.worker_queues.insert(config.name(), config);
        self
    }

    pub fn new<A>(task_store: S, context_fn: A) -> Self
    where
        A: Fn() -> C + Send + Sync + 'static,
    {
        Self {
            _lifetime: PhantomData,
            context_fn: Arc::new(context_fn),
            task_store,
            //       executor,
            task_registry: BTreeMap::new(),
            schedule_registry: BTreeMap::new(),
            //startup_registry: BTreeMap::new(),
            queue_tasks: BTreeMap::new(),
            worker_queues: BTreeMap::new(),
        }
    }

    pub fn register_task_type<TL>(mut self) -> Self
    where
        TL: TaskLike<Context = C>,
    {
        self.queue_tasks
            .entry(TL::QUEUE_NAME)
            .or_default()
            .push(TL::TASK_NAME);

        self.task_registry
            .insert(TL::TASK_NAME, Arc::new(deserialize_and_run_task::<TL>));

        self
    }

    pub async fn register_recurring_task_type<RT, E>(mut self, executor: &mut E) -> Self
    where
        RT: RecurringTask<Context = C>,
        for<'e> &'e mut E: Executor<'e, Database = D>,
    {
        self.schedule_registry
            .insert(RT::TASK_NAME, Arc::new(next_schedule::<RT>));

        S::enqueue_exec(&mut *executor, RT::default()).await;

        self.register_task_type::<RT>()
    }

    pub async fn start<F>(self, shutdown_signal: F) -> Result<JoinHandle<()>, WorkerPoolError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        /*
        for (task_name, enqueue_recurring_task_fn) in self.startup_registry.clone().into_iter() {
            let future = enqueue_recurring_task_fn(PhantomData);
            let result = future(&mut *executor).await;

            match result.map_err(|err| {
                WorkerPoolError::FailedToEnqueueRecurring(task_name.to_string(), err)
            }) {
                Ok(task_id) => {
                    tracing::info!(
                        "enqueued recurring task `{}` with task_id `{}`",
                        task_name,
                        task_id.unwrap_or("None".to_string())
                    );
                }
                Err(err) => {
                    tracing::error!("failed to enqueue recurring task: {err}");
                }
            }
        }
        */

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

                let mut worker: Worker<C, S, D> = Worker::new(
                    worker_name.clone(),
                    queue_config.clone(),
                    self.context_fn.clone(),
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
    FailedToEnqueueRecurring(String, TaskStoreError),
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
    task.next_schedule()
        .map_err(TaskExecError::SchedulingFailed)
}

/*
fn enqueue_recurring_task<RT, S, D, E>(
    _db: PhantomData<D>,
) -> Pin<Box<dyn Fn(&mut E) -> dyn Future<Output = Result<Option<String>, TaskStoreError>>>>
where
    RT: RecurringTask,
    D: Database,
    S: TaskStore<D>,
    for<'e> &'e mut E: Executor<'e, Database = D>,
{
    Box::pin(move |executor| S::enqueue_exec(executor, RT::default()))
}
*/
