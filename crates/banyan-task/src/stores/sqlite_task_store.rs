use std::time::Duration;

use async_trait::async_trait;
use sqlx::pool::PoolConnection;
use sqlx::{Acquire, Sqlite, SqliteConnection, SqlitePool};
use time::OffsetDateTime;

use crate::task_store::TaskStore;
use crate::{
    Task, TaskInstanceBuilder, TaskLike, TaskState, TaskStoreError, TaskStoreMetrics,
    TASK_EXECUTION_TIMEOUT,
};

#[derive(Clone)]
pub struct SqliteTaskStore {
    pool: SqlitePool,
}

impl SqliteTaskStore {
    async fn connect(&self) -> Result<PoolConnection<Sqlite>, TaskStoreError> {
        Ok(self.pool.clone().acquire().await?)
    }

    pub async fn is_present<T: TaskLike>(
        conn: &mut SqliteConnection,
        task: &T,
    ) -> Result<bool, TaskStoreError> {
        let unique_key = task.unique_key();
        if unique_key.is_none() {
            let res = sqlx::query_scalar!(
                "SELECT 1 FROM background_tasks WHERE task_name = $1",
                T::TASK_NAME
            )
            .fetch_optional(&mut *conn)
            .await?;
            return Ok(res.unwrap_or(0) == 1);
        }
        let res = sqlx::query_scalar!(
            "SELECT 1 FROM background_tasks WHERE task_name = $1 AND unique_key = $2",
            T::TASK_NAME,
            unique_key
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(res.unwrap_or(0) == 1)
    }

    async fn is_key_present(
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

    pub async fn metrics(&self) -> Result<TaskStoreMetrics, TaskStoreError> {
        let mut conn = self.connect().await?;
        let mut query_base = self.metrics_query();
        let query = query_base.build_query_as::<SqliteTaskStoreMetrics>();
        let metrics = query.fetch_one(&mut *conn).await?;
        Ok(metrics.into())
    }

    fn metrics_query(&self) -> sqlx::query_builder::QueryBuilder<'static, sqlx::Sqlite> {
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

    pub async fn get_task(&self, id: String) -> Result<Task, TaskStoreError> {
        let task = sqlx::query_as!(Task, r#"SELECT * FROM background_tasks WHERE id = $1"#, id)
            .fetch_one(&self.pool)
            .await?;

        Ok(task)
    }

    async fn create(
        conn: &mut SqliteConnection,
        task: TaskInstanceBuilder,
    ) -> Result<Option<String>, TaskStoreError> {
        if let Some(ukey) = &task.unique_key {
            // right now if we encounter a unique key that is already present in the DB we simply
            // don't queue the new instance of that task, the old one will have a bit of priority
            // due to its age.
            if SqliteTaskStore::is_key_present(&mut *conn, ukey, &task.task_name).await? {
                return Ok(None);
            }
        }

        let background_task_id: String = sqlx::query_scalar!(
            r#"
                INSERT INTO background_tasks
                (
                    task_name, queue_name, unique_key, payload, 
                    current_attempt, maximum_attempts, state,
                    original_task_id, scheduled_to_run_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING id;
            "#,
            task.task_name,
            task.queue_name,
            task.unique_key,
            task.payload,
            task.current_attempt,
            task.maximum_attempts,
            task.state,
            task.original_task_id,
            task.scheduled_to_run_at,
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(Some(background_task_id))
    }
}

#[async_trait]
impl TaskStore for SqliteTaskStore {
    type Connection = SqliteConnection;

    async fn enqueue<T: TaskLike>(
        connection: &mut Self::Connection,
        task: T,
    ) -> Result<Option<String>, TaskStoreError> {
        let task = TaskInstanceBuilder::for_task(task).await?;
        let background_task_id = Self::create(&mut *connection, task).await?;
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
                      AND state IN ($2, $3)
                      AND DATETIME(scheduled_to_run_at) <= DATETIME('now')
                   ORDER BY DATETIME(scheduled_to_run_at) ASC, DATETIME(scheduled_at) ASC
                   LIMIT 1;"#,
            queue_name,
            TaskState::New,
            TaskState::Retry,
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
                           state = $1
                       WHERE id = $2;"#,
                TaskState::InProgress,
                id,
            )
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        let mut conn = self.connect().await?;
        let timed_out_start_threshold = time::OffsetDateTime::now_utc() - TASK_EXECUTION_TIMEOUT;
        let pending_retry_tasks = sqlx::query_scalar!(
            r#"SELECT id FROM background_tasks
                   WHERE state IN ($1, $2)
                      AND DATETIME(started_at) <= DATETIME($3)
                   ORDER BY DATETIME(started_at) ASC
                   LIMIT 10;"#,
            TaskState::InProgress,
            TaskState::Retry,
            timed_out_start_threshold,
        )
        .fetch_all(&mut *conn)
        .await;
        conn.close().await?;

        // if this query fails or any of our rescheduling fails, we still want to process our task,
        // let these retry again sometime in the future. Ideally we'd randomly shuffle some of
        // these to prevent head-of-line blocking by a single poison task.
        //
        // Ideally we'd also take one of these ourselves if next_task_id is none but that is some
        // extra complicated logic we don't need right now
        if let Ok(task_ids) = pending_retry_tasks {
            for id in task_ids.into_iter() {
                let mut conn = self.connect().await?;
                // we don't care of these fail either, but we'll stop attempting to retry them once
                // we hit an error. Something else can handle the trouble

                let state_update_res = sqlx::query!(
                    r#"UPDATE background_tasks
                          SET
                              finished_at = DATETIME('now'),
                              state = $1
                          WHERE id = $2"#,
                    TaskState::TimedOut,
                    id,
                )
                .execute(&mut *conn)
                .await;

                conn.close().await?;

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
        let mut connection = self.pool.acquire().await?;
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
        let mut connection = self.pool.acquire().await?;
        let mut transaction = connection.begin().await?;

        // We only care about tasks that are capable of being retried so some filters here allow
        // irrelevant attempts to quickly and silently be ignored.
        let maybe_retried_task = sqlx::query_as!(
            Task,
            r#"SELECT * FROM background_tasks
                   WHERE id = $1
                       AND state IN ($2, $3)
                       AND current_attempt < maximum_attempts;"#,
            id,
            TaskState::Error,
            TaskState::TimedOut,
        )
        .fetch_optional(&mut *transaction)
        .await?;

        let retried_task = match maybe_retried_task {
            Some(rt) => rt,
            None => return Ok(None),
        };

        let backoff_time_secs = 30u64 * 3u64.saturating_pow(retried_task.current_attempt as u32);
        let next_run_at = OffsetDateTime::now_utc() + Duration::from_secs(backoff_time_secs);

        let task = TaskInstanceBuilder::from_task_instance(retried_task)
            .await
            .run_at(next_run_at);
        let new_task_id = Self::create(&mut transaction, task).await?;

        transaction.commit().await?;

        Ok(new_task_id)
    }

    async fn update_state(&self, id: String, new_state: TaskState) -> Result<(), TaskStoreError> {
        let mut connection = self.pool.acquire().await?;

        // this could probably use some protection against invalid state transitions but I'll leave
        // that as future work for now.
        sqlx::query!(
            r#"UPDATE background_tasks
                   SET state = $1
                   WHERE id = $2;"#,
            new_state,
            id,
        )
        .execute(&mut *connection)
        .await?;

        Ok(())
    }

    async fn schedule_next(
        &self,
        task_id: String,
        next_schedule: OffsetDateTime,
    ) -> Result<Option<String>, TaskStoreError> {
        let task_instance = self.get_task(task_id).await?;

        let task = TaskInstanceBuilder::from_task_instance(task_instance)
            .await
            .reset_task()
            .run_at(next_schedule);

        let mut conn = self.pool.acquire().await?;
        let new_task_id = Self::create(&mut conn, task).await?;
        Ok(new_task_id)
    }

    async fn get_task_in_state(
        &self,
        task_name: &str,
        states: Vec<TaskState>,
    ) -> Result<Option<Task>, TaskStoreError> {
        let mut query_builder =
            sqlx::QueryBuilder::new("SELECT * FROM background_tasks WHERE task_name =");
        query_builder.push_bind(task_name);
        query_builder.push(" AND state IN (");
        let mut separated_values = query_builder.separated(", ");
        for state in states {
            separated_values.push_bind(state);
        }
        query_builder.push(");");

        let query = query_builder
            .build_query_as::<Task>()
            .persistent(false)
            .fetch_optional(&self.pool)
            .await;

        match query {
            Ok(Some(res)) => Ok(Some(res)),
            Ok(None) => Ok(None),
            Err(err) => Err(TaskStoreError::DatabaseError(err)),
        }
    }
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
    use crate::tests::TestTask;
    use crate::TaskLikeExt;

    #[tokio::test]
    async fn reschedule_tasks_work() {
        let (task_store, task_id) = singleton_task_store().await;
        let next_time = OffsetDateTime::now_utc() + Duration::from_secs(60);
        let task_id = task_store
            .schedule_next(task_id.expect("task should be created"), next_time)
            .await
            .expect("schedule_next");
        let metrics = task_store.metrics().await.unwrap();
        assert_ne!(task_id, None, "Task should have been rescheduled");
        assert_eq!(metrics.total, 2);
        assert_eq!(metrics.scheduled, 1);
    }

    #[tokio::test]
    async fn update_state_works() {
        let (task_store, _task_id) = singleton_task_store().await;
        let task = TestTask;
        let mut conn = task_store.connect().await.unwrap();
        let task_id = task
            .enqueue::<SqliteTaskStore>(&mut conn)
            .await
            .expect("enqueue")
            .expect("task create_from_taskd");
        let update_result = task_store
            .update_state(task_id, TaskState::InProgress)
            .await;

        assert!(update_result.is_ok(), "Update state failed");
        let metrics = task_store.metrics().await.unwrap();
        assert_eq!(metrics.in_progress, 1, "Task state not updated correctly");
    }

    #[tokio::test]
    async fn error_tasks_are_retried() {
        let (task_store, _task_id) = singleton_task_store().await;
        let task = TestTask;
        let mut conn = task_store.connect().await.unwrap();
        let task_id = task
            .enqueue::<SqliteTaskStore>(&mut conn)
            .await
            .expect("enqueue")
            .expect("task created");
        let _ = task_store
            .update_state(task_id.clone(), TaskState::Error)
            .await;
        let retry_result = task_store.retry(task_id).await;

        assert!(retry_result.is_ok(), "Retry failed");
        assert_ne!(retry_result.unwrap(), None, "task should have been retried");
        let metrics = task_store.metrics().await.unwrap();
        assert_eq!(metrics.retried, 1, "Task not retried correctly");
    }

    #[tokio::test]
    async fn timeout_tasks_are_retried() {
        let (task_store, _task_id) = singleton_task_store().await;
        let task = TestTask;
        let mut conn = task_store.connect().await.unwrap();
        let task_id = task
            .enqueue::<SqliteTaskStore>(&mut conn)
            .await
            .expect("enqueue")
            .expect("task create_from_taskd");
        let _ = task_store
            .update_state(task_id.clone(), TaskState::TimedOut)
            .await;
        let retry_result = task_store.retry(task_id).await;

        assert!(retry_result.is_ok(), "Retry failed");
        assert_ne!(retry_result.unwrap(), None, "task should have been retried");
        let metrics = task_store.metrics().await.unwrap();
        assert_eq!(metrics.retried, 1, "Task not retried correctly");
    }

    #[tokio::test]
    async fn non_error_tasks_are_not_retried() {
        let (task_store, _task_id) = singleton_task_store().await;
        let task = TestTask;
        let mut conn = task_store.connect().await.unwrap();
        let task_id = task
            .enqueue::<SqliteTaskStore>(&mut conn)
            .await
            .expect("enqueue")
            .expect("task create_from_taskd");
        let retry_result = task_store.retry(task_id).await;
        assert!(retry_result.is_ok(), "Retry failed");
        assert_eq!(
            retry_result.unwrap(),
            None,
            "task should not have been retried"
        );
        let metrics = task_store.metrics().await.unwrap();
        assert_eq!(metrics.retried, 0, "task should not have been retried");
    }

    #[tokio::test]
    async fn empty_store_works() {
        let task_store = empty_task_store().await;
        let metrics = task_store.metrics().await.unwrap();
        assert_eq!(metrics, TaskStoreMetrics::default());
    }

    pub async fn singleton_task_store() -> (SqliteTaskStore, Option<String>) {
        let task_store = empty_task_store().await;
        let mut conn = task_store.connect().await.unwrap();
        let task_id = TestTask
            .enqueue::<SqliteTaskStore>(&mut conn)
            .await
            .expect("enqueue");
        (task_store, task_id)
    }

    pub async fn empty_task_store() -> SqliteTaskStore {
        let db = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("db setup");
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .expect("db setup");
        SqliteTaskStore::new(db)
    }
}
