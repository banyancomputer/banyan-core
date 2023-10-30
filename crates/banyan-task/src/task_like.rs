use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{CurrentTask, TaskStore, TaskStoreError};

#[async_trait]
pub trait TaskLike: Serialize + DeserializeOwned + Sync + Send + 'static {
    // todo: rename MAX_ATTEMPTS
    const MAX_RETRIES: i64 = 3;

    const QUEUE_NAME: &'static str = "default";

    const TASK_NAME: &'static str;

    type Error: std::error::Error;
    type Context: Clone + Send + 'static;

    async fn run(&self, task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error>;

    async fn unique_key(&self) -> Option<String> {
        None
    }
}

#[async_trait]
pub trait TaskLikeExt {
    async fn enqueue<S: TaskStore>(
        self,
        pool: &mut S::Pool,
    ) -> Result<Option<String>, TaskStoreError>;

    async fn enqueue_with_connection<S: TaskStore>(
        self,
        pool: &mut S::Connection,
    ) -> Result<Option<String>, TaskStoreError>;
}

#[async_trait]
impl<T> TaskLikeExt for T
where
    T: TaskLike,
{
    async fn enqueue<S: TaskStore>(
        self,
        pool: &mut S::Pool,
    ) -> Result<Option<String>, TaskStoreError> {
        S::enqueue(pool, self).await
    }

    async fn enqueue_with_connection<S: TaskStore>(
        self,
        connection: &mut S::Connection,
    ) -> Result<Option<String>, TaskStoreError> {
        S::enqueue_with_connection(connection, self).await
    }
}
