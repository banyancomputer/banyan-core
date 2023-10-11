use std::fmt::{self, Debug, Display, Formatter};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use uuid::Uuid;

mod test_task;

pub use test_task::TestTask;

use crate::workers::{CurrentTask, TaskQueueError, TaskStore};

#[async_trait]
pub trait TaskLike: Serialize + DeserializeOwned + Sync + Send + 'static {
    // todo: rename MAX_ATTEMPTS
    const MAX_RETRIES: usize = 3;

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
        connection: &mut S::Connection,
    ) -> Result<Option<TaskId>, TaskQueueError>;
}

#[async_trait]
impl<T> TaskLikeExt for T
where
    T: TaskLike,
{
    async fn enqueue<S: TaskStore>(
        self,
        connection: &mut S::Connection,
    ) -> Result<Option<TaskId>, TaskQueueError> {
        S::enqueue(connection, self).await
    }
}

#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TaskId(Uuid);

impl Debug for TaskId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TaskId").field(&self.0).finish()
    }
}

impl Display for TaskId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for TaskId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<TaskId> for Uuid {
    fn from(value: TaskId) -> Self {
        value.0
    }
}
