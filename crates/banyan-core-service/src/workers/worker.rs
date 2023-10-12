#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::workers::panic_safe_future::PanicSafeFuture;
use crate::workers::{
    CurrentTask, ExecuteTaskFn, QueueConfig, StateFn, Task, TaskExecError, TaskState, TaskStore,
    TaskStoreError, MAXIMUM_CHECK_DELAY,
};

pub struct Worker<Context, S>
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
    pub fn new(
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

    pub async fn run(&self, task: Task) -> Result<(), WorkerError> {
        let task_info = CurrentTask::new(&task);

        let deserialize_and_run_task_fn = self
            .task_registry
            .get(task.task_name.as_str())
            .ok_or_else(|| WorkerError::UnregisteredTaskName(task.task_name))?
            .clone();

        let safe_runner = PanicSafeFuture::wrap({
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
                self.store
                    .update_state(task.id, TaskState::Panicked)
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
                self.store
                    .update_state(task.id, TaskState::Complete)
                    .await
                    .map_err(WorkerError::UpdateTaskStatusFailed)?;
            }
            Err(err) => {
                tracing::error!("task failed with error: {err}");

                self.store
                    .errored(task.id, TaskExecError::ExecutionFailed(err.to_string()))
                    .await
                    .map_err(WorkerError::RetryTaskFailed)?;
            }
        }

        Ok(())
    }

    pub async fn run_tasks(&mut self) -> Result<(), WorkerError> {
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

            let next_task = self
                .store
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
                    if let Ok(_signaled) =
                        tokio::time::timeout(MAXIMUM_CHECK_DELAY, ss.changed()).await
                    {
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
