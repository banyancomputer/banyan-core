use serde::{Deserialize, Serialize};

use crate::database::models::{Notification, NotificationSeverity};

#[derive(Deserialize, Serialize)]
pub struct ApiNotification {
    pub id: String,
    pub user_id: String,
    pub dismissable: bool,
    pub message: String,
    pub message_key: String,
    pub severity: NotificationSeverity,
    pub created_at: i64,
}

impl From<Notification> for ApiNotification {
    fn from(notification: Notification) -> Self {
        Self {
            id: notification.id,
            user_id: notification.user_id,
            dismissable: notification.dismissable,
            message: notification.message,
            message_key: notification.message_key,
            severity: notification.severity,
            created_at: notification.created_at.unix_timestamp(),
        }
    }
}
