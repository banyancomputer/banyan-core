#![allow(dead_code)]
use std::collections::BTreeMap;

use crate::panic_safe_future::PanicSafeFuture;
use crate::{
    CurrentTask, CurrentTaskError, ExecuteTaskFn, NextScheduleFn, QueueConfig, StateFn, Task,
    TaskExecError, TaskState, TaskStore, TaskStoreError, MAXIMUM_CHECK_DELAY,
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
    schedule_registry: BTreeMap<&'static str, NextScheduleFn>,
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
        schedule_registry: BTreeMap<&'static str, NextScheduleFn>,
        shutdown_signal: Option<tokio::sync::watch::Receiver<()>>,
    ) -> Self {
        Self {
            name,
            queue_config,
            context_data_fn,
            store,
            task_registry,
            schedule_registry,
            shutdown_signal,
        }
    }

    #[tracing::instrument(level = "error", skip_all, fields(task_name = %task.task_name, task_id = %task.id))]
    pub async fn run(&self, task: Task) -> Result<(), WorkerError> {
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

        let task_info = CurrentTask::try_from(&task).map_err(WorkerError::CantMakeCurrent)?;
        let deserialize_and_run_task_fn = self
            .task_registry
            .get(task.task_name.as_str())
            .ok_or_else(|| WorkerError::UnregisteredTaskName(task.task_name.clone()))?
            .clone();

        let context = (self.context_data_fn)();
        let payload = task.payload.clone();
        let safe_runner = PanicSafeFuture::wrap(async move {
            deserialize_and_run_task_fn(task_info, payload, context).await
        });

        // an error here occurs only when the task panicks, deserialization and regular task
        // execution errors are handled next
        //
        // todo: should note the task as having panicked if that's why this failed. There is also a
        // chance that the worker is corrupted in some way by the panic so I should set a flag on
        // this worker and handle two consecutive panics as a worker problem. The second task
        // triggering the panic should be presumed innocent and restored to a runnable state.
        match safe_runner.await {
            Ok(task_result) => {
                match task_result {
                    Ok(()) => {
                        self.store
                            .completed(task.id.clone())
                            .await
                            .map_err(WorkerError::UpdateTaskStatusFailed)?;
                        self.schedule_if_needed(&task).await
                    }
                    Err(err) => {
                        tracing::error!("task failed with error: {err}");
                        match self
                            .store
                            .errored(
                                task.id.clone(),
                                TaskExecError::ExecutionFailed(err.to_string()),
                            )
                            .await
                            .map_err(WorkerError::RetryTaskFailed)?
                        {
                            // not retried
                            None => self.schedule_if_needed(&task).await,
                            // retried
                            _ => Ok(()),
                        }
                    }
                }
            }
            Err(_) => {
                tracing::error!("task panicked");
                // todo: save panic message into the task.error and save it back to the memory
                // store somehow...
                self.store
                    .update_state(task.id.clone(), TaskState::Panicked)
                    .await
                    .map_err(WorkerError::UpdateTaskStatusFailed)
            }
        }
    }

    async fn schedule_if_needed(&self, task: &Task) -> Result<(), WorkerError> {
        if let Some(get_next_schedule) = self.schedule_registry.get(task.task_name.as_str()) {
            if let Ok(Some(next_schedule)) = get_next_schedule(task.payload.clone()) {
                return self
                    .store
                    .schedule_next(task.id.clone(), next_schedule)
                    .await
                    .map(|_| ())
                    .map_err(|err| {
                        WorkerError::ScheduleFailed(format!(
                            "unable to schedule {}, err: {}",
                            task.task_name, err
                        ))
                    });
            }
        }

        Ok(())
    }

    pub async fn run_tasks(&mut self) -> Result<(), WorkerError> {
        let relevant_task_names: Vec<&'static str> = self.task_registry.keys().cloned().collect();
        // While there are still tasks in the queue
        loop {
            if let Some(task) = self
                .store
                .next(self.queue_config.name(), &relevant_task_names)
                .await
                .map_err(WorkerError::StoreUnavailable)?
            {
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
    #[error("failed to generate current task info for task execution: {0}")]
    CantMakeCurrent(CurrentTaskError),

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

    #[error("task deserialization failed: {0}")]
    DeserializationFailed(#[from] serde_json::Error),

    #[error("task schedule failed: {0}")]
    ScheduleFailed(String),
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores::{singleton_task_store, SqliteTaskStore};
    use crate::task_like::tests::ScheduleTestTask;
    use crate::tests::TestTask;
    use crate::TaskLike;
    use futures::FutureExt;
    use std::sync::Arc;
    use tokio::sync::watch;
    const WORKER_NAME: &str = "default";
    const TEST_CONTEXT: TestContext = TestContext {};
    impl Worker<TestContext, SqliteTaskStore> {
        async fn next_task(&self, task_name: &str) -> Task {
            self.store
                .next(WORKER_NAME, &[task_name])
                .await
                .map_err(WorkerError::StoreUnavailable)
                .expect("could not get task from db")
                .expect("could not create task instance")
        }
        async fn get_task(&self, id: String) -> Option<Task> {
            match self.store.get_task(id).await {
                Ok(t) => Some(t),
                Err(_err) => None,
            }
        }
    }
    #[derive(Clone)]
    struct TestContext {}
    fn create_registry() -> BTreeMap<&'static str, ExecuteTaskFn<TestContext>> {
        let mut task_registry = BTreeMap::new();

        let test_task_fn: ExecuteTaskFn<TestContext> =
            Arc::new(|_task, _payload, _context| async { Ok(()) }.boxed());

        let schedule_test_task_fn: ExecuteTaskFn<TestContext> =
            Arc::new(|_task, _payload, _context| {
                async {
                    Err(TaskExecError::ExecutionFailed(
                        "failed with error".to_string(),
                    ))
                }
                .boxed()
            });

        task_registry.insert(TestTask::TASK_NAME, test_task_fn);
        task_registry.insert(ScheduleTestTask::TASK_NAME, schedule_test_task_fn);
        task_registry
    }
    fn create_worker(
        ctx: &'static TestContext,
        task_store: SqliteTaskStore,
    ) -> Worker<TestContext, SqliteTaskStore> {
        let queue_config = QueueConfig::new(WORKER_NAME).with_worker_count(1);
        let task_registry = create_registry();
        let context_data_fn = Arc::new(move || ctx.clone());
        let (_, inner_shutdown_rx) = watch::channel(());

        Worker::new(
            WORKER_NAME.to_string(),
            queue_config.clone(),
            context_data_fn.clone(),
            task_store,
            task_registry.clone(),
            BTreeMap::new(),
            Some(inner_shutdown_rx.clone()),
        )
    }

    #[tokio::test]
    async fn test_worker_run_unregistered_task() {
        let (task_store, _task_id) = singleton_task_store().await;
        let worker = create_worker(&TEST_CONTEXT, task_store);
        let task_name = "UnregisteredTask";
        let mut task = worker.next_task(task_name).await;
        task.task_name = String::from(task_name);
        let result = worker.run(task).await;

        assert!(
            result.is_err(),
            "Worker run should fail with unregistered task"
        );
        if let WorkerError::UnregisteredTaskName(err) = result.unwrap_err() {
            assert_eq!(err, task_name);
        } else {
            panic!("did not return the correct error type")
        }
    }

    // currently has issues with accessing the in-memory database
    #[ignore]
    #[tokio::test]
    async fn test_worker_run_successful_task() {
        let (task_store, _task_id) = singleton_task_store().await;
        let worker = create_worker(&TEST_CONTEXT, task_store);
        let task = worker.next_task(TestTask::TASK_NAME).await;
        let result = worker.run(task).await;
        assert!(result.is_ok(), "Worker run failed with a valid task");
    }
}
