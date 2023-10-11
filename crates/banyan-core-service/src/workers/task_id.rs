use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use uuid::Uuid;

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
