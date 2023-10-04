use serde::Serialize;

use super::EmailMessage;

#[derive(Serialize)]
pub struct ReachingStorageLimit; 

impl EmailMessage for ReachingStorageLimit {
    fn subject() -> String {
        "Your Storage is Almost Full".to_string()
    }

    fn template_name() -> &'static str {
        "reaching_storage_limit"
    }
}