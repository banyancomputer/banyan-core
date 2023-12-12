use std::time::Duration;

use async_trait::async_trait;
use sqlx::{Acquire, SqliteConnection, SqlitePool};
use time::OffsetDateTime;

use crate::{
    Task, TaskInstanceBuilder, TaskLike, TaskState, TaskStore, TaskStoreError, TaskStoreMetrics,
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

    async fn next(
        &self,
        queue_name: &str,
        _task_names: &[&str],
    ) -> Result<Option<Task>, TaskStoreError> {
        let mut transaction = self.pool.clone().begin().await?;
        // todo: need to dynamically build up the task_names portion of this query since sqlx
        // doesn't support generation of IN queries or have a concept of arrays for sqlite.l

        let next_task_id: Option<String> = sqlx::query_scalar!(
            r#"SELECT id FROM background_tasks
                   WHERE queue_name = $1
                      AND state IN ('new', 'retry')
                      AND DATETIME(scheduled_to_run_at) <= DATETIME('now')
                   ORDER BY DATETIME(scheduled_to_run_at) ASC, DATETIME(scheduled_at) ASC
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
        let mut connection = self.pool.clone().acquire().await?;

        let timed_out_start_threshold = time::OffsetDateTime::now_utc() - TASK_EXECUTION_TIMEOUT;
        let pending_retry_tasks = sqlx::query_scalar!(
            r#"SELECT id FROM background_tasks
                   WHERE state IN ('in_progress', 'retry')
                      AND DATETIME(started_at) <= DATETIME($1)
                   ORDER BY DATETIME(started_at) ASC
                   LIMIT 10;"#,
            timed_out_start_threshold,
        )
        .fetch_all(&mut *connection)
        .await;

        connection.close().await?;

        // if this query fails or any of our rescheduling fails, we still want to process our task,
        // let these retry again sometime in the future. Ideally we'd randomly shuffle some of
        // these to prevent head-of-line blocking by a single poison task.
        //
        // Ideally we'd also take one of these ourselves if next_task_id is none but that is some
        // extra complicated logic we don't need right now
        if let Ok(task_ids) = pending_retry_tasks {
            for id in task_ids.into_iter() {
                let mut connection = self.pool.clone().acquire().await?;
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
                .execute(&mut *connection)
                .await;

                connection.close().await?;

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
        let mut connection = self.pool.clone().acquire().await?;
        let chosen_task = sqlx::query_as!(
            Task,
            "SELECT * FROM background_tasks WHERE id = $1;",
            chosen_task_id
        )
        .fetch_one(&mut *connection)
        .await?;
        connection.close().await?;
        Ok(Some(chosen_task))
    }

    async fn retry(&self, id: String) -> Result<Option<String>, TaskStoreError> {
        let mut connection = self.pool.clone().acquire().await?;
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

    async fn metrics(&self) -> Result<TaskStoreMetrics, TaskStoreError> {
        let mut connection = self.pool.clone().acquire().await?;
        let mut query_base = metrics_query();
        let query = query_base.build_query_as::<SqliteTaskStoreMetrics>();
        let metrics = query.fetch_one(&mut *connection).await?;
        Ok(metrics.into())
    }

    async fn task_metrics(
        &self,
        task_name: &'static str,
    ) -> Result<TaskStoreMetrics, TaskStoreError> {
        let mut connection = self.pool.clone().acquire().await?;
        let mut query_base = metrics_query();
        let query = query_base
            .push(" WHERE task_name =  ")
            .push_bind(task_name)
            .build_query_as::<SqliteTaskStoreMetrics>();
        let metrics = query.fetch_one(&mut *connection).await?;
        Ok(metrics.into())
    }

    async fn queue_metrics(
        &self,
        queue_name: &'static str,
    ) -> Result<TaskStoreMetrics, TaskStoreError> {
        let mut connection = self.pool.clone().acquire().await?;
        let mut query_base = metrics_query();
        let query = query_base
            .push(" WHERE queue_name =  ")
            .push_bind(queue_name)
            .build_query_as::<SqliteTaskStoreMetrics>();
        let metrics = query.fetch_one(&mut *connection).await?;
        Ok(metrics.into())
    }

    async fn update_state(&self, id: String, new_state: TaskState) -> Result<(), TaskStoreError> {
        let mut connection = self.pool.clone().acquire().await?;

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

fn metrics_query() -> sqlx::query_builder::QueryBuilder<'static, sqlx::Sqlite> {
    sqlx::query_builder::QueryBuilder::new(
        r#"SELECT
               COUNT(*) AS total,
               COALESCE(SUM(CASE WHEN state = 'new' THEN 1 ELSE 0 END), 0) AS new,
               COALESCE(SUM(CASE WHEN state = 'in_progress' THEN 1 ELSE 0 END), 0) AS in_progress,
               COALESCE(SUM(CASE WHEN state = 'panicked' THEN 1 ELSE 0 END), 0) AS panicked,
               COALESCE(SUM(CASE WHEN state = 'retry' THEN 1 ELSE 0 END), 0) AS retried,
               COALESCE(SUM(CASE WHEN state = 'cancelled' THEN 1 ELSE 0 END), 0) AS cancelled,
               COALESCE(SUM(CASE WHEN state = 'error' THEN 1 ELSE 0 END), 0) AS errored,
               COALESCE(SUM(CASE WHEN state = 'complete' THEN 1 ELSE 0 END), 0) AS completed,
               COALESCE(SUM(CASE WHEN state = 'timed_out' THEN 1 ELSE 0 END), 0) AS timed_out,
               COALESCE(SUM(CASE WHEN state = 'dead' THEN 1 ELSE 0 END), 0) AS dead,
               COALESCE(SUM(CASE WHEN DATETIME(scheduled_to_run_at) <= DATETIME('now') THEN 1 ELSE 0 END), 0) AS scheduled,
               COALESCE(SUM(CASE WHEN DATETIME(scheduled_to_run_at) > DATETIME('now') THEN 1 ELSE 0 END), 0) AS scheduled_future
            FROM background_tasks"#,
    )
}

#[derive(sqlx::FromRow)]
struct SqliteTaskStoreMetrics {
    total: i32,
    new: i32,
    in_progress: i32,
    panicked: i32,
    retried: i32,
    cancelled: i32,
    errored: i32,
    completed: i32,
    timed_out: i32,
    dead: i32,
    scheduled: i32,
    scheduled_future: i32,
}

impl From<SqliteTaskStoreMetrics> for TaskStoreMetrics {
    fn from(m: SqliteTaskStoreMetrics) -> Self {
        Self {
            total: m.total,
            new: m.new,
            in_progress: m.in_progress,
            panicked: m.panicked,
            retried: m.retried,
            cancelled: m.cancelled,
            errored: m.errored,
            completed: m.completed,
            timed_out: m.timed_out,
            dead: m.dead,
            scheduled: m.scheduled,
            scheduled_future: m.scheduled_future,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::tests::{default_task_store_metrics, TestTask};
    use crate::TaskLikeExt;

    #[tokio::test]
    async fn empty_metrics() {
        let task_store = empty_task_store().await;
        let metrics = task_store.metrics().await.unwrap();
        assert_eq!(metrics, default_task_store_metrics());
    }

    #[tokio::test]
    async fn singleton_metrics() {
        let task_store = singleton_task_store().await;
        let metrics = task_store.metrics().await.unwrap();
        assert_eq!(metrics.total, 1);
        assert_eq!(metrics.new, 1);
        assert_eq!(metrics.scheduled, 1);
        assert_eq!(metrics.scheduled_future, 0);
    }

    #[tokio::test]
    async fn empty_queue_metrics() {
        let task_store = empty_task_store().await;
        let metrics = task_store.queue_metrics("default").await.unwrap();
        assert_eq!(metrics, default_task_store_metrics());
    }

    #[tokio::test]
    async fn singleton_queue_metrics() {
        let task_store = singleton_task_store().await;
        let metrics = task_store
            .queue_metrics(TestTask::QUEUE_NAME)
            .await
            .unwrap();
        assert_eq!(metrics.total, 1);
        assert_eq!(metrics.new, 1);
        assert_eq!(metrics.scheduled, 1);
        assert_eq!(metrics.scheduled_future, 0);
    }

    #[tokio::test]
    async fn empty_task_metrics() {
        let task_store = empty_task_store().await;
        let metrics = task_store.task_metrics("default").await.unwrap();
        assert_eq!(metrics, default_task_store_metrics());
    }

    #[tokio::test]
    async fn singleton_task_metrics() {
        let task_store = singleton_task_store().await;
        let metrics = task_store.task_metrics(TestTask::TASK_NAME).await.unwrap();
        assert_eq!(metrics.total, 1);
        assert_eq!(metrics.new, 1);
        assert_eq!(metrics.scheduled, 1);
        assert_eq!(metrics.scheduled_future, 0);
    }

    async fn singleton_task_store() -> SqliteTaskStore {
        let task_store = empty_task_store().await;
        TestTask
            .enqueue::<SqliteTaskStore>(&mut task_store.pool.clone())
            .await
            .expect("enqueue");
        task_store
    }

    async fn empty_task_store() -> SqliteTaskStore {
        let db_conn = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("db setup");
        sqlx::migrate!("./migrations")
            .run(&db_conn)
            .await
            .expect("db setup");
        SqliteTaskStore::new(db_conn)
    }
}
