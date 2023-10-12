use async_trait::async_trait;
use chrono::offset::Utc;
use sqlx::SqlitePool;

use crate::workers::{TASK_EXECUTION_TIMEOUT, Task, TaskLike, TaskState, TaskStore, TaskStoreError};

#[derive(Clone)]
pub struct SqliteTaskStore {
    pool: SqlitePool,
}

impl SqliteTaskStore {
    async fn is_key_present(pool: &SqlitePool, key: &str) -> Result<bool, TaskStoreError> {
        let query_res = sqlx::query_scalar!("SELECT 1 FROM background_tasks WHERE unique_key = $1;", key)
            .fetch_optional(&*pool)
            .await
            .map_err(|err| TaskStoreError::ConnectionFailure(err.to_string()))?;

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

        if let Some(ukey) = &unique_key {
            // right now if we encounter a unique key that is already present in the DB we simply
            // don't queue the new instance of that task, the old one will have a bit of priority
            // due to its age.
            if SqliteTaskStore::is_key_present(pool, ukey).await? {
                return Ok(None);
            }
        }

        let payload = serde_json::to_string(&task)
            .map_err(TaskStoreError::EncodeFailed)?;

        let task_name = T::TASK_NAME.to_string();
        let queue_name = T::QUEUE_NAME.to_string();

        let background_task_id: String = sqlx::query_scalar!(
                r#"INSERT INTO background_tasks (task_name, unique_key, queue_name, payload, maximum_attempts)
                       VALUES ($1, $2, $3, $4, $5)
                       RETURNING id;"#,
                task_name,
                unique_key,
                queue_name,
                payload,
                T::MAX_RETRIES,
            )
            .fetch_one(&*pool)
            .await
            .map_err(|err| TaskStoreError::ConnectionFailure(err.to_string()))?;

        Ok(Some(background_task_id))
    }

    async fn next(&self, queue_name: &str, _task_names: &[&str]) -> Result<Option<Task>, TaskStoreError> {
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
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| TaskStoreError::ConnectionFailure(err.to_string()))?;

        let timed_out_start_threshold = Utc::now() - TASK_EXECUTION_TIMEOUT;
        let pending_retry_tasks = sqlx::query_scalar!(
            r#"SELECT id FROM background_tasks
                 WHERE state = 'in_progress'
                    AND started_at <= $1
                ORDER BY started_at ASC
                LIMIT 10;"#,
            timed_out_start_threshold,
        )
        .fetch_all(&self.pool)
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

                if let Err(_) = state_update_res {
                    break;
                }

                if let Err(_) = self.retry(id).await {
                    break;
                }
            }
        }

        let chosen_task_id = match next_task_id {
            Some(nti) => nti,
            None => return Ok(None),
        };

        // todo: should add a worker identifier when picking up a job for both logging/tracking as
        // well as future directed clean up
        sqlx::query!(
            r#"UPDATE background_tasks
                   SET
                       state = 'in_progress',
                       started_at = DATETIME('now')
                   WHERE id = ?;"#,
            chosen_task_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|err| TaskStoreError::ConnectionFailure(err.to_string()))?;

        // pull the full current version of the task
        let chosen_task = sqlx::query_as!(Task, "SELECT * FROM background_tasks WHERE id = $1;", chosen_task_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| TaskStoreError::ConnectionFailure(err.to_string()))?;

        Ok(Some(chosen_task))
    }

    async fn retry(&self, _id: String) -> Result<Option<String>, TaskStoreError> {
        // leaving as a todo for now
        Ok(None)
    }

    async fn update_state(&self, id: String, new_state: TaskState) -> Result<(), TaskStoreError> {
        // this could probably use some protection against invalid state transitions but I'll leave
        // that as future work for now.
        sqlx::query!(
            r#"UPDATE background_tasks
                SET state = $2
                WHERE id = $1;"#,
            id,
            new_state,
        )
        .execute(&self.pool)
        .await
        .map_err(|err| TaskStoreError::ConnectionFailure(err.to_string()))?;

        Ok(())
    }
}
