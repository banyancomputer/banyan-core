use async_trait::async_trait;
use chrono::offset::Utc;
use sqlx::{Acquire, SqliteConnection, SqlitePool};

use crate::workers::{
    Task, TaskLike, TaskState, TaskStore, TaskStoreError, TASK_EXECUTION_TIMEOUT,
};

#[derive(Clone)]
pub struct SqliteTaskStore {
    pool: SqlitePool,
}

impl SqliteTaskStore {
    async fn checked_create(
        conn: &mut SqliteConnection,
        unique_key: Option<&str>,
        task_name: &str,
        queue_name: &str,
        current_attempt: i64,
        maximum_attempts: i64,
        payload: String,
        previous_task_id: Option<String>,
    ) -> Result<Option<String>, TaskStoreError> {
        if let Some(ukey) = &unique_key {
            // right now if we encounter a unique key that is already present in the DB we simply
            // don't queue the new instance of that task, the old one will have a bit of priority
            // due to its age.
            if SqliteTaskStore::is_key_present(conn, ukey, task_name).await? {
                return Ok(None);
            }
        }

        let background_task_id: String = sqlx::query_scalar!(
                r#"INSERT INTO background_tasks (previous_id, task_name, unique_key, queue_name, payload, current_attempt, maximum_attempts)
                       VALUES ($1, $2, $3, $4, $5, $6, $7)
                       RETURNING id;"#,
                previous_task_id,
                task_name,
                unique_key,
                queue_name,
                payload,
                current_attempt,
                maximum_attempts,
            )
            .fetch_one(&mut *conn)
            .await?;

        Ok(Some(background_task_id))
    }

    async fn is_key_present(
        conn: &mut SqliteConnection,
        key: &str,
        task_name: &str,
    ) -> Result<bool, TaskStoreError> {
        let query_res =
            sqlx::query_scalar!("SELECT 1 FROM background_tasks WHERE unique_key = $1 AND task_name = $2;", key, task_name)
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
    type Connection = SqlitePool;

    async fn enqueue<T: TaskLike>(
        pool: &mut Self::Connection,
        task: T,
    ) -> Result<Option<String>, TaskStoreError> {
        let unique_key = task.unique_key().await;

        let mut connection = pool.acquire().await?;
        let mut transaction = connection.begin().await?;

        let task_name = T::TASK_NAME.to_string();
        let queue_name = T::QUEUE_NAME.to_string();

        let payload = serde_json::to_string(&task).map_err(TaskStoreError::EncodeFailed)?;

        let background_task_id = SqliteTaskStore::checked_create(
            &mut transaction,
            unique_key.as_ref().map(|x| x.as_str()),
            &task_name,
            &queue_name,
            0,
            T::MAX_RETRIES,
            payload,
            None,
        )
        .await?;

        transaction.commit().await?;

        Ok(background_task_id)
    }

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

        let timed_out_start_threshold = Utc::now() - TASK_EXECUTION_TIMEOUT;
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
        .fetch_optional(&self.pool)
        .await?;

        let retried_task = match maybe_retried_task {
            Some(rt) => rt,
            None => return Ok(None),
        };

        let new_task_id = SqliteTaskStore::checked_create(
            &mut transaction,
            retried_task.unique_key.as_ref().map(|x| x.as_str()),
            &retried_task.task_name,
            &retried_task.queue_name,
            retried_task.current_attempt + 1,
            retried_task.maximum_attempts,
            retried_task.payload.to_string(),
            Some(retried_task.id),
        )
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
