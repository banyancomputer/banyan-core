mod ga_release;
mod payment_failed;
mod product_invoice;
mod reaching_storage_limit;
mod scheduled_maintenance;

#[allow(unused)]
pub use ga_release::GaReleaseEmailTask;
#[allow(unused)]
pub use payment_failed::PaymentFailedEmailTask;
#[allow(unused)]
pub use product_invoice::ProductInvoiceEmailTask;
#[allow(unused)]
pub use reaching_storage_limit::ReachingStorageLimitEmailTask;
#[allow(unused)]
pub use scheduled_maintenance::ScheduledMaintenanceEmailTask;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::email::config::EmailConfig;
use crate::email::error::EmailError;
use crate::email::message::EmailMessage;

#[derive(Clone)]
pub struct EmailTaskContext {
    db_pool: SqlitePool,
    email_config: EmailConfig,
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum EmailTaskError {
    #[error("email error: {0}")]
    EmailError(#[from] EmailError),

    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
}

#[allow(dead_code)]
impl EmailTaskContext {
    pub fn db_pool(&self) -> &SqlitePool {
        &self.db_pool
    }

    pub fn email_config(&self) -> &EmailConfig {
        &self.email_config
    }

    pub fn new(db_pool: SqlitePool, email_config: EmailConfig) -> Self {
        Self {
            db_pool,
            email_config,
        }
    }
}

/// Tests recipient for filtering conditions against the provided context
/// # Arguments
/// * `user_id` - The account id of the user to get the email address for
/// * `ctx` - The context to use for the task
/// # Returns
/// * `Result<bool, EmailTaskError>` - Whether or not the user should be sent an email
pub async fn should_send_email_message(
    user_id: Uuid,
    ctx: &EmailTaskContext,
) -> Result<bool, EmailTaskError> {
    let mut db_conn = ctx.db_pool.acquire().await?;
    let user_id_string = user_id.to_string();

    // If our last email we sent to the user resulting in a status of unsubscribe we should not send the message
    let unsubscribed = sqlx::query_scalar!(
        r#"SELECT
                CASE
                    WHEN last_email.state = 'unsubscribed' THEN 1
                    ELSE 0
                END
            FROM (
                SELECT state
                FROM emails e
                WHERE e.user_id = $1
                ORDER BY e.sent_at DESC
                LIMIT 1
            ) AS last_email;"#,
        user_id_string
    )
    .fetch_optional(&mut *db_conn)
    .await?;
    if unsubscribed == Some(1) {
        tracing::info!("user {user_id_string} has unsubscribed");
        return Ok(false);
    }

    // If the last three emails we sent to a user resulted in a delivery failure we should not send the message
    let failures = sqlx::query_scalar!(
        r#"SELECT
                SUM(CASE WHEN state = 'failed' THEN 1 ELSE 0 END) AS failures
            FROM (
                SELECT state
                FROM emails e
                WHERE e.user_id = $1
                ORDER BY e.sent_at DESC
                LIMIT 3 
            ) AS last_three_emails;"#,
        user_id_string
    )
    .fetch_one(&mut *db_conn)
    .await?;
    if failures >= Some(3) {
        tracing::info!("user {user_id_string} has had 3 or more consecutive failed emails");
        return Ok(false);
    }

    // If we have sent 3 or more email to the user which have been marked as spam we should not send the message
    sqlx::query!(
        r#"INSERT OR IGNORE INTO email_stats (user_id) VALUES ($1);"#,
        user_id_string
    )
    .execute(&mut *db_conn)
    .await?;
    let complaints = sqlx::query_scalar!(
        r#"SELECT
                complained
            FROM email_stats e
            WHERE e.user_id = $1
            LIMIT 1"#,
        user_id_string
    )
    .fetch_one(&mut *db_conn)
    .await?;
    if complaints >= 3 {
        tracing::info!("user {user_id_string} has had 3 or more spam complaints");
        return Ok(false);
    }
    Ok(true)
}

/// Goes through the process of sending an email message to a user. It
/// - Determines the recipient address
/// - Records the outgoing message in the context and asigns it a message id
/// - Sends the message -- Note, if this fails the message will not be retried!
/// # Arguments
/// * `user_id` - The account id of the user to get the email address for
/// * `message` - The message to send
/// * `ctx` - The context to use for the task
/// # Returns
/// * `Result<(), EmailTaskError>` - The message id of the recorded message
pub async fn send_email_message(
    user_id: Uuid,
    message: &impl EmailMessage,
    ctx: &EmailTaskContext,
) -> Result<(), EmailTaskError> {
    let mut db_conn = ctx.db_pool.acquire().await?;
    let transport = ctx.email_config.transport()?;
    let from_address = ctx.email_config.from();
    let mailgun_test_mode = ctx.email_config.test_mode();
    let user_id_string = user_id.to_string();
    let message_type_name = message.type_name();

    // Get the recipient address -- do this first to prevent side effects from this failing
    let recipient_address = sqlx::query_scalar!(
        r#"SELECT u.email as "email!"
        FROM users u
        WHERE u.id = $1;"#,
        user_id_string
    )
    .fetch_one(&mut *db_conn)
    .await?;

    // Record the outgoing message
    let new_email_id = sqlx::query_scalar!(
        r#"INSERT INTO emails (user_id, type)
         VALUES ($1, $2)
         RETURNING id"#,
        user_id_string,
        message_type_name
    )
    .fetch_one(&mut *db_conn)
    .await?;
    let message_id = new_email_id.parse::<Uuid>().expect("invalid uuid");

    // Send the email -- capture errors to prevent the task from being retried
    let send_result = message.send(
        &transport,
        from_address,
        &recipient_address,
        message_id,
        mailgun_test_mode,
    );
    match send_result {
        Ok(_) => {}
        Err(error) => {
            tracing::error!("email failed to send: {}", error);
        }
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use banyan_task::{CurrentTask, TaskLike};
    use time::OffsetDateTime;

    use super::*;
    use crate::database::test_helpers::*;

    const USER_ID: &str = "00000000-0000-0000-0000-000000000000";
    const USER_EMAIL: &str = "user@user.email";

    /// Return a base context and a test account id
    pub async fn test_setup() -> (EmailTaskContext, Uuid, CurrentTask) {
        (
            email_task_context().await,
            Uuid::parse_str(USER_ID).expect("account id parse"),
            CurrentTask::default(),
        )
    }

    pub async fn count_sent_emails(ctx: &EmailTaskContext) -> i32 {
        let mut db_conn = ctx.db_pool().acquire().await.unwrap();
        sqlx::query_scalar!(
            r#"SELECT
                COUNT(*) AS sent
            FROM emails e
            WHERE e.user_id = $1
                AND e.state = 'sent'"#,
            USER_ID
        )
        .fetch_one(&mut *db_conn)
        .await
        .expect("db setup")
    }

    #[tokio::test]
    async fn success() -> Result<(), EmailTaskError> {
        let ctx = email_task_context().await;
        example_email_task(ctx.clone()).await?;
        let email_count = count_sent_emails(&ctx).await;
        assert_eq!(email_count, 1);
        Ok(())
    }

    #[tokio::test]
    async fn unsubscribed() -> Result<(), EmailTaskError> {
        let ctx = email_task_context().await;
        // Create one unsubscribed email as the last email sent
        let now = OffsetDateTime::now_utc();
        let _later = unsubscribed_email(&ctx, now).await;
        example_email_task(ctx.clone()).await?;
        let email_count = count_sent_emails(&ctx).await;
        assert_eq!(email_count, 0);
        Ok(())
    }

    #[tokio::test]
    async fn unsubscribed_then_sent() -> Result<(), EmailTaskError> {
        let ctx = email_task_context().await;
        // One unsubscribed email then one delivered email as the last email sent
        let now = OffsetDateTime::now_utc();
        let later = unsubscribed_email(&ctx, now).await;
        let _later = delivered_email(&ctx, later).await;
        example_email_task(ctx.clone()).await?;
        let email_count = count_sent_emails(&ctx).await;
        assert_eq!(email_count, 1);
        Ok(())
    }

    #[tokio::test]
    async fn failure() -> Result<(), EmailTaskError> {
        let ctx = email_task_context().await;
        // Threee failed emails in a row as the last three emails sent
        let now = OffsetDateTime::now_utc();
        let later = failed_email(&ctx, now).await;
        let later = failed_email(&ctx, later).await;
        let _later = failed_email(&ctx, later).await;
        example_email_task(ctx.clone()).await?;
        let email_count = count_sent_emails(&ctx).await;
        assert_eq!(email_count, 0);
        Ok(())
    }

    #[tokio::test]
    async fn failure_then_sent() -> Result<(), EmailTaskError> {
        let ctx = email_task_context().await;
        // Threee failed emails in a row and then one delivered email
        let now = OffsetDateTime::now_utc();
        let later = failed_email(&ctx, now).await;
        let later = failed_email(&ctx, later).await;
        let later = failed_email(&ctx, later).await;
        let _later = delivered_email(&ctx, later).await;
        example_email_task(ctx.clone()).await?;
        let email_count = count_sent_emails(&ctx).await;
        assert_eq!(email_count, 1);
        Ok(())
    }

    #[tokio::test]
    async fn complaint() -> Result<(), EmailTaskError> {
        let ctx = email_task_context().await;
        let now = OffsetDateTime::now_utc();
        // Three complaints interspersed with delivered emails
        let later = complained_email(&ctx, now).await;
        let later = delivered_email(&ctx, later).await;
        let later = complained_email(&ctx, later).await;
        let later = delivered_email(&ctx, later).await;
        let _later = complained_email(&ctx, later).await;
        let email_count = count_sent_emails(&ctx).await;
        assert_eq!(email_count, 0);
        Ok(())
    }

    async fn example_email_task(ctx: EmailTaskContext) -> Result<(), EmailTaskError> {
        // Use GaReleaseEmailTask as a stand in for any email task here
        let task = GaReleaseEmailTask::new(Uuid::parse_str(USER_ID).unwrap());
        task.run(CurrentTask::default(), ctx).await?;
        Ok(())
    }

    async fn delivered_email(ctx: &EmailTaskContext, when: OffsetDateTime) -> OffsetDateTime {
        let mut db_conn = ctx.db_pool().acquire().await.unwrap();
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, user_id, state, type)
            VALUES ($1, $2, 'delivered', 'na')"#,
            when,
            USER_ID
        )
        .execute(&mut *db_conn)
        .await
        .expect("db setup");
        sqlx::query!(
            r#"UPDATE email_stats
            SET delivered = delivered + 1
            WHERE user_id = $1"#,
            USER_ID
        )
        .execute(&mut *db_conn)
        .await
        .expect("db setup");

        when + std::time::Duration::from_secs(1)
    }

    async fn failed_email(ctx: &EmailTaskContext, when: OffsetDateTime) -> OffsetDateTime {
        let mut db_conn = ctx.db_pool().acquire().await.unwrap();
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, user_id, state, type)
            VALUES ($1, $2, 'failed', 'na')"#,
            when,
            USER_ID
        )
        .execute(&mut *db_conn)
        .await
        .expect("db setup");
        sqlx::query!(
            r#"UPDATE email_stats
            SET failed = failed + 1
            WHERE user_id = $1"#,
            USER_ID
        )
        .execute(&mut *db_conn)
        .await
        .expect("db setup");

        when + std::time::Duration::from_secs(1)
    }

    async fn complained_email(ctx: &EmailTaskContext, when: OffsetDateTime) -> OffsetDateTime {
        let mut db_conn = ctx.db_pool().acquire().await.unwrap();
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, user_id, state, type)
            VALUES ($1, $2, 'complained', 'na')"#,
            when,
            USER_ID
        )
        .execute(&mut *db_conn)
        .await
        .expect("db setup");
        sqlx::query!(
            r#"UPDATE email_stats
            SET complained = complained + 1
            WHERE user_id = $1"#,
            USER_ID
        )
        .execute(&mut *db_conn)
        .await
        .expect("db setup");

        when + std::time::Duration::from_secs(1)
    }

    async fn unsubscribed_email(ctx: &EmailTaskContext, when: OffsetDateTime) -> OffsetDateTime {
        let mut db_conn = ctx.db_pool().acquire().await.unwrap();
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, user_id, state, type)
            VALUES ($1, $2, 'unsubscribed', 'na')"#,
            when,
            USER_ID
        )
        .execute(&mut *db_conn)
        .await
        .expect("db setup");
        sqlx::query!(
            r#"UPDATE email_stats
            SET unsubscribed = unsubscribed + 1
            WHERE user_id = $1"#,
            USER_ID
        )
        .execute(&mut *db_conn)
        .await
        .expect("db setup");

        when + std::time::Duration::from_secs(1)
    }

    async fn email_task_context() -> EmailTaskContext {
        let db_conn = setup_database().await;

        sqlx::query!(
            r#"INSERT INTO users (id, email, display_name)
            VALUES ($1, $2, $3);"#,
            USER_ID,
            USER_EMAIL,
            "test user"
        )
        .execute(&db_conn)
        .await
        .expect("db setup");

        sqlx::query!(
            r#"INSERT INTO email_stats (user_id)
            VALUES ($1)"#,
            USER_ID
        )
        .execute(&db_conn)
        .await
        .expect("db setup");

        EmailTaskContext::new(
            db_conn,
            EmailConfig::new(None, "test@test.email", false).unwrap(),
        )
    }
}
