use serde::Serialize;
use time::OffsetDateTime;

use crate::database::{models::NotificationSeverity, DatabaseConnection};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub dismissable: bool,
    pub message: String,
    pub message_key: String,
    pub severity: NotificationSeverity,
    pub created_at: OffsetDateTime,
}

impl Notification {
    /// Checks whether the provided notification ID is owned by the provided user ID. This will return
    /// false when the user IDs don't match, if the notification doesn't exist (the user inherently
    /// doesn't own an unknown ID), or if the notification has already been deleted.
    pub async fn is_owned_by_user_id(
        conn: &mut DatabaseConnection,
        notification_id: &str,
        user_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let found_notification = sqlx::query_scalar!(
            r#"
                SELECT id 
                FROM notifications 
                WHERE id = $1 
                AND user_id = $2;
            "#,
            notification_id,
            user_id,
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(found_notification.is_some())
    }

    pub async fn delete(
        conn: &mut DatabaseConnection,
        notification_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                DELETE FROM notifications
                WHERE id = $1;
            "#,
            notification_id,
        )
        .execute(&mut *conn)
        .await
        .map(|_| ())
    }
}
