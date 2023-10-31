use std::time::Duration;

use async_trait::async_trait;
use sqlx::{Acquire, SqliteConnection, SqlitePool};
use time::OffsetDateTime;

use crate::{
    Task, TaskInstanceBuilder, TaskLike, TaskState, TaskStore, TaskStoreError,
    TASK_EXECUTION_TIMEOUT,
};

#[derive(Clone)]
pub struct SqliteTaskStore {
    pool: SqlitePool,
}

impl SqliteTaskStore {
    pub async fn is_key_present(
        conn: &mut SqliteConnection,
        key: &str,
        task_name: &str,
    ) -> Result<bool, TaskStoreError> {
        let query_res = sqlx::query_scalar!(
            "SELECT 1 FROM background_tasks WHERE unique_key = $1 AND task_name = $2;",
            key,
            task_name
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(query_res.is_some())
    }

    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaskStore for SqliteTaskStore {
    type Pool = SqlitePool;
    type Connection = SqliteConnection;

    async fn enqueue<T: TaskLike>(
        pool: &mut Self::Pool,
        task: T,
    ) -> Result<Option<String>, TaskStoreError> {
        let mut connection = pool.acquire().await?;
        let mut transaction = connection.begin().await?;

        let background_task_id = TaskInstanceBuilder::for_task(task)
            .await?
            .create(&mut transaction)
            .await?;

        transaction.commit().await?;

        Ok(background_task_id)
    }

    async fn enqueue_with_connection<T: TaskLike>(
        connection: &mut Self::Connection,
        task: T,
    ) -> Result<Option<String>, TaskStoreError> {
        let background_task_id = TaskInstanceBuilder::for_task(task)
            .await?
            .create(&mut *connection)
            .await?;

        Ok(background_task_id)
    }

    #[tracing::instrument(level = debug, skip(self))]
    async fn next(
        &self,
        queue_name: &str,
        _task_names: &[&str],
    ) -> Result<Option<Task>, TaskStoreError> {
        let mut connection = self.pool.acquire().await?;
        let mut transaction = connection.begin().await?;

        // todo: need to dynamically build up the task_names portion of this query since sqlx
        // doesn't support generation of IN queries or have a concept of arrays for sqlite.l

        let next_task_id: Option<String> = sqlx::query_scalar!(
            r#"SELECT id FROM background_tasks
                   WHERE queue_name = $1
                      AND state IN ('new', 'retry')
                      AND scheduled_to_run_at <= DATETIME('now')
                   ORDER BY scheduled_to_run_at ASC, scheduled_at ASC
                   LIMIT 1;"#,
            queue_name,
        )
        .fetch_optional(&mut *transaction)
        .await?;

        // If we found it claim it for this worker in the same transaction
        //
        // todo: should add a worker identifier when picking up a job for both logging/tracking as
        // well as future directed clean up
        if let Some(ref id) = next_task_id {
            sqlx::query!(
                r#"UPDATE background_tasks
                       SET started_at = DATETIME('now'),
                           state = 'in_progress'
                       WHERE id = $1;"#,
                id,
            )
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        tracing::debug!(?next_task_id, "queried for next task to run");

        let timed_out_start_threshold = time::OffsetDateTime::now_utc() - TASK_EXECUTION_TIMEOUT;
        let pending_retry_tasks = sqlx::query_scalar!(
            r#"SELECT id FROM background_tasks
                   WHERE state IN ('in_progress', 'retry')
                      AND started_at <= $1
                   ORDER BY started_at ASC
                   LIMIT 10;"#,
            timed_out_start_threshold,
        )
        .fetch_all(&mut *connection)
        .await;

        // if this query fails or any of our rescheduling fails, we still want to process our task,
        // let these retry again sometime in the future. Ideally we'd randomly shuffle some of
        // these to prevent head-of-line blocking by a single poison task.
        //
        // Ideally we'd also take one of these ourselves if next_task_id is none but that is some
        // extra complicated logic we don't need right now
        if let Ok(task_ids) = pending_retry_tasks {
            for id in task_ids.into_iter() {
                // we don't care of these fail either, but we'll stop attempting to retry them once
                // we hit an error. Something else can handle the trouble

                let state_update_res = sqlx::query!(
                    r#"UPDATE background_tasks
                          SET
                              finished_at = DATETIME('now'),
                              state = 'timed_out'
                          WHERE id = $1"#,
                    id,
                )
                .execute(&self.pool)
                .await;

                if state_update_res.is_err() {
                    break;
                }

                if self.retry(id).await.is_err() {
                    break;
                }
            }
        }

        tracing::debug!("cleaned up retryable tasks");

        let chosen_task_id = match next_task_id {
            Some(nti) => nti,
            None => return Ok(None),
        };

        // pull the full current version of the task
        let chosen_task = sqlx::query_as!(
            Task,
            "SELECT * FROM background_tasks WHERE id = $1;",
            chosen_task_id
        )
        .fetch_one(&self.pool)
        .await?;

        tracing::info!(?task, "located task to be run");

        Ok(Some(chosen_task))
    }

    async fn retry(&self, id: String) -> Result<Option<String>, TaskStoreError> {
        let mut connection = self.pool.acquire().await?;
        let mut transaction = connection.begin().await?;

        // We only care about tasks that are capable of being retried so some filters here allow
        // irrelevant attempts to quickly and silently be ignored.
        let maybe_retried_task = sqlx::query_as!(
            Task,
            r#"SELECT * FROM background_tasks
                   WHERE id = $1
                       AND state IN ('error', 'timed_out')
                       AND current_attempt < maximum_attempts;"#,
            id
        )
        .fetch_optional(&mut *transaction)
        .await?;

        let retried_task = match maybe_retried_task {
            Some(rt) => rt,
            None => return Ok(None),
        };

        let backoff_time_secs = 30u64 * 3u64.saturating_pow(retried_task.current_attempt as u32);
        let next_run_at = OffsetDateTime::now_utc() + Duration::from_secs(backoff_time_secs);

        let new_task_id = TaskInstanceBuilder::from_task_instance(retried_task)
            .await
            .run_at(next_run_at)
            .create(&mut transaction)
            .await?;

        transaction.commit().await?;

        Ok(new_task_id)
    }

    async fn update_state(&self, id: String, new_state: TaskState) -> Result<(), TaskStoreError> {
        let mut connection = self.pool.acquire().await?;

        // this could probably use some protection against invalid state transitions but I'll leave
        // that as future work for now.
        sqlx::query!(
            r#"UPDATE background_tasks
                   SET state = $2
                   WHERE id = $1;"#,
            id,
            new_state,
        )
        .execute(&mut *connection)
        .await?;

        Ok(())
    }
}
