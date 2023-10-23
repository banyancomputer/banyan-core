use serde::{Deserialize, Serialize};

use super::EmailMessage;

#[derive(Serialize, Deserialize)]
pub struct ReachingStorageLimit {
    pub(crate) current_usage: usize,
    pub(crate) max_usage: usize,
}

impl EmailMessage for ReachingStorageLimit {
    const SUBJECT: &'static str = "You're reaching your storage limit";
    const TEMPLATE_NAME: &'static str = "reaching_storage_limit";
    const TYPE_NAME: &'static str = "reaching_storage_limit";
}
