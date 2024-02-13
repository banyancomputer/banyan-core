use serde::Serialize;
use time::OffsetDateTime;

use crate::database::models::NotificationSeverity;
use crate::database::DatabaseConnection;

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
    pub async fn get(
        conn: &mut DatabaseConnection,
        notification_id: &str,
        user_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
                SELECT id, user_id, dismissable, message, message_key, severity as 'severity: NotificationSeverity', created_at 
                FROM notifications 
                WHERE id = $1 
                AND user_id = $2;
            "#,
            notification_id,
            user_id,
        )
        .fetch_optional(&mut *conn)
        .await
    }

    pub async fn delete(
        conn: &mut DatabaseConnection,
        notification_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                DELETE FROM notifications
                WHERE id = $1
                AND dismissable = true;
            "#,
            notification_id,
        )
        .execute(&mut *conn)
        .await
        .map(|_| ())
    }
}
