mod ga_release;

pub use ga_release::GaReleaseEmailTask;

use sqlx::SqlitePool;
use tracing;
use uuid::Uuid;

use crate::db::models::CreatedResource;
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
    #[error("the task encountered an email error: {0}")]
    EmailError(#[from] EmailError),
    #[error("the task encountered a sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
}

#[allow(dead_code)]
impl EmailTaskContext {
    pub fn new(db_pool: SqlitePool, email_config: EmailConfig) -> Self {
        Self {
            db_pool,
            email_config,
        }
    }

    pub fn db_pool(&self) -> &SqlitePool {
        &self.db_pool
    }

    pub fn email_config(&self) -> &EmailConfig {
        &self.email_config
    }
}

/// Tests recipient for filtering conditions against the provided context
/// # Arguments
/// * `account_id` - The account id of the user to get the email address for
/// * `ctx` - The context to use for the task
/// # Returns
/// * `Result<bool, EmailTaskError>` - Whether or not the user should be sent an email
pub async fn should_send_email_message(
    account_id: Uuid,
    ctx: &EmailTaskContext,
) -> Result<bool, EmailTaskError> {
    let mut db_conn = ctx.db_pool.acquire().await?;
    let account_id_string = account_id.to_string();

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
                WHERE e.account_id = $1
                ORDER BY e.sent_at DESC
                LIMIT 1
            ) AS last_email;"#,
        account_id_string
    )
    .fetch_optional(&mut *db_conn)
    .await?;
    if unsubscribed == Some(1) {
        tracing::info!("user {account_id_string} has unsubscribed");
        return Ok(false);
    }

    // If the last three emails we sent to a user resulted in a delivery failure we should not send the message
    let failures = sqlx::query_scalar!(
        r#"SELECT
                COUNT(*) AS failures
            FROM (
                SELECT state
                FROM emails e
                WHERE e.account_id = $1
                ORDER BY e.sent_at DESC
                LIMIT 3 
            ) AS last_three_emails
            WHERE
                last_three_emails.state = 'failed';"#,
        account_id_string
    )
    .fetch_optional(&mut *db_conn)
    .await?;
    if failures >= Some(3) {
        tracing::info!("user {account_id_string} has had 3 or more consecutive failed emails");
        return Ok(false);
    }

    // If we have sent 3 or more email to the user which have been marked as spam we should not send the message
    let complaints = sqlx::query_scalar!(
        r#"SELECT
                COUNT(*) AS complaints
            FROM emails e
            WHERE e.account_id = $1
                AND e.state = 'complained'
            LIMIT 3"#,
        account_id_string
    )
    .fetch_optional(&mut *db_conn)
    .await?;
    if complaints >= Some(3) {
        tracing::info!("user {account_id_string} has had 3 or more spam complaints");
        return Ok(false);
    }
    Ok(true)
}

/// Goes through the process of sending an email message to a user. It
/// - Determines the recipient address
/// - Records the outgoing message in the context and asigns it a message id
/// - Sends the message -- Note, if this fails the message will not be retried!
/// # Arguments
/// * `account_id` - The account id of the user to get the email address for
/// * `message` - The message to send
/// * `ctx` - The context to use for the task
/// # Returns
/// * `Result<(), EmailTaskError>` - The message id of the recorded message
pub async fn send_email_message(
    account_id: Uuid,
    message: &impl EmailMessage,
    ctx: &EmailTaskContext,
) -> Result<(), EmailTaskError> {
    let mut db_conn = ctx.db_pool.acquire().await?;
    let transport = ctx.email_config.transport()?;
    let from_address = ctx.email_config.from();
    let mailgun_test_mode = ctx.email_config.test_mode();
    let account_id_string = account_id.to_string();
    let message_type_name = message.type_name();

    // Get the recipient address -- do this first to prevent side effects from this failing
    let recipient_address = sqlx::query_scalar!(
        r#"SELECT u.email as "email!"
        FROM users u
        JOIN accounts a ON u.id = a.userId
        WHERE a.id = $1;"#,
        account_id_string
    )
    .fetch_one(&mut *db_conn)
    .await?;

    // Record the outgoing message
    let created_resource = sqlx::query_as!(
        CreatedResource,
        r#"INSERT INTO emails (account_id, type)
         VALUES ($1, $2)
         RETURNING id"#,
        account_id_string,
        message_type_name
    )
    .fetch_one(&mut *db_conn)
    .await?;
    let message_id = created_resource.id.parse::<Uuid>().expect("invalid uuid");

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
