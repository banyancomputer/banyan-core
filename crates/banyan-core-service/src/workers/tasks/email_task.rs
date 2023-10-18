#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::models::CreatedResource;
use crate::email::config::EmailConfig;
use crate::email::error::EmailError;
use crate::email::message::EmailMessage;
use crate::workers::CurrentTask;
use crate::workers::TaskLike;

#[derive(Deserialize, Serialize)]
pub struct EmailTask<M> {
    account_id: Uuid,
    message: M,
    config: EmailConfig,
}

impl<M> EmailTask<M>
where
    M: EmailMessage,
{
    pub fn new(account_id: Uuid, message: M, config: EmailConfig) -> Self {
        Self {
            account_id,
            message,
            config,
        }
    }
}

#[async_trait]
impl<M> TaskLike for EmailTask<M>
where
    M: EmailMessage,
{
    const TASK_NAME: &'static str = "email_task";

    type Error = EmailTaskError;
    type Context = sqlx::SqlitePool;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let transport = self.config.transport()?;

        // Filter out innapropriate emails
        let mut connection = ctx.acquire().await?;
        let account_id = self.account_id.to_string();

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
            account_id
        )
        .fetch_optional(&mut *connection)
        .await?;
        if unsubscribed == Some(1) {
            tracing::info!("the user has unsubscribed from emails");
            return Ok(());
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
            account_id
        )
        .fetch_optional(&mut *connection)
        .await?;
        if failures >= Some(3) {
            tracing::info!("the user has had too many sequential delivery failures");
            return Ok(());
        }

        // If we have sent 3 or more email to the user which have been marked as spam we should not send the message
        let complaints = sqlx::query_scalar!(
            r#"SELECT
                COUNT(*) AS complaints
            FROM emails e
            WHERE e.account_id = $1
                AND e.state = 'complained'
            LIMIT 3"#,
            account_id
        )
        .fetch_optional(&mut *connection)
        .await?;
        if complaints >= Some(3) {
            tracing::info!("the user has marked too many emails as spam");
            return Ok(());
        }

        // Get the recipient address
        let recipient_address = sqlx::query_scalar!(
            r#"SELECT u.email as "email!"
            FROM users u
            JOIN accounts a ON u.id = a.userId
            WHERE a.id = $1;"#,
            account_id
        )
        .fetch_one(&mut *connection)
        .await?;

        // Create a new record for the message
        let type_name = self.message.type_name();
        let created_resource = sqlx::query_as!(
            CreatedResource,
            r#"INSERT INTO emails (account_id, type)
            VALUES ($1, $2)
            RETURNING id"#,
            account_id,
            type_name
        )
        .fetch_one(&mut *connection)
        .await?;
        let message_id = created_resource.id.parse::<Uuid>().expect("invalid uuid");

        // Send the email -- capture errors to prevent the task from being retried
        let send_result = self.message.send(
            &transport,
            self.config.from(),
            &recipient_address,
            message_id,
            self.config.test_mode(),
        );
        match send_result {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("email failed to send: {}", error);
            }
        }

        return Ok(());
    }
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum EmailTaskError {
    #[error("the task encountered an email error: {0}")]
    EmailError(#[from] EmailError),
    #[error("the task encountered a sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    const ACCOUNT_ID: &str = "00000000-0000-0000-0000-000000000000";
    const USER_EMAIL: &str = "user@user.email";

    async fn email_task(ctx: SqlitePool) -> Result<(), EmailTaskError> {
        let task = EmailTask::new(
            Uuid::parse_str(ACCOUNT_ID).unwrap(),
            crate::email::message::GaRelease {},
            EmailConfig::new(None, "test@test.test", false).unwrap(),
        );
        task.run(CurrentTask::default(), ctx).await?;
        Ok(())
    }

    #[tokio::test]
    async fn success() -> Result<(), EmailTaskError> {
        let mut ctx = context().await;
        email_task(ctx.clone()).await?;
        let email_count = sent_emails(&mut ctx).await;
        assert_eq!(email_count, 1);
        Ok(())
    }

    #[tokio::test]
    async fn unsubscribed() -> Result<(), EmailTaskError> {
        let mut ctx = unsubscribed_context().await;
        email_task(ctx.clone()).await?;
        let email_count = sent_emails(&mut ctx).await;
        assert_eq!(email_count, 0);
        Ok(())
    }

    #[tokio::test]
    async fn unsubscribed_then_sent() -> Result<(), EmailTaskError> {
        let mut ctx = unsubscribed_context().await;
        // Add one delivered email
        let now = chrono::Utc::now() + chrono::Duration::seconds(1);
        sqlx::query!(
            r#"INSERT INTO emails (account_id, sent_at, state, type)
            VALUES ($1, $2, 'delivered', 'na')"#,
            ACCOUNT_ID,
            now
        )
        .execute(&ctx)
        .await
        .expect("db setup");
        email_task(ctx.clone()).await?;
        let email_count = sent_emails(&mut ctx).await;
        assert_eq!(email_count, 1);
        Ok(())
    }

    #[tokio::test]
    async fn failure() -> Result<(), EmailTaskError> {
        let mut ctx = failure_context().await;
        email_task(ctx.clone()).await?;
        let email_count = sent_emails(&mut ctx).await;
        assert_eq!(email_count, 0);
        Ok(())
    }

    #[tokio::test]
    async fn failure_then_sent() -> Result<(), EmailTaskError> {
        let mut ctx = failure_context().await;
        // Add one delivered email
        let now = chrono::Utc::now() + chrono::Duration::seconds(10);
        sqlx::query!(
            r#"INSERT INTO emails (account_id, sent_at, state, type)
            VALUES ($1, $2, 'delivered', 'na')"#,
            ACCOUNT_ID,
            now
        )
        .execute(&ctx)
        .await
        .expect("db setup");
        email_task(ctx.clone()).await?;
        let email_count = sent_emails(&mut ctx).await;
        assert_eq!(email_count, 1);
        Ok(())
    }

    #[tokio::test]
    async fn complaint() -> Result<(), EmailTaskError> {
        let mut ctx = complaint_context().await;
        email_task(ctx.clone()).await?;
        let email_count = sent_emails(&mut ctx).await;
        assert_eq!(email_count, 0);
        Ok(())
    }

    async fn unsubscribed_context() -> SqlitePool {
        let pool = context().await;

        // Add one delivered email
        let now = chrono::Utc::now();
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'delivered', 'na')"#,
            now,
            ACCOUNT_ID,
        )
        .execute(&pool)
        .await
        .expect("db setup");

        // Most recent must be unsubscribed
        let now = now + chrono::Duration::seconds(1);
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'unsubscribed', 'na')"#,
            now,
            ACCOUNT_ID,
        )
        .execute(&pool)
        .await
        .expect("db setup");

        pool
    }

    async fn failure_context() -> SqlitePool {
        let pool = context().await;

        // Add one delivered email
        let now = chrono::Utc::now();
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'delivered', 'na')"#,
            now,
            ACCOUNT_ID
        )
        .execute(&pool)
        .await
        .expect("db setup");

        // Add three failed emails
        let now = now + chrono::Duration::seconds(1);
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'failed', 'na')"#,
            now,
            ACCOUNT_ID
        )
        .execute(&pool)
        .await
        .expect("db setup");

        let now = now + chrono::Duration::seconds(1);
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'failed', 'na')"#,
            now,
            ACCOUNT_ID
        )
        .execute(&pool)
        .await
        .expect("db setup");

        let now = now + chrono::Duration::seconds(1);
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'failed', 'na')"#,
            now,
            ACCOUNT_ID
        )
        .execute(&pool)
        .await
        .expect("db setup");

        pool
    }

    async fn complaint_context() -> SqlitePool {
        let pool = context().await;

        // Add one delivered email
        let now = chrono::Utc::now();
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'delivered', 'na')"#,
            now,
            ACCOUNT_ID
        )
        .execute(&pool)
        .await
        .expect("db setup");

        // Add three complaints
        let now = now + chrono::Duration::seconds(1);
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'complained', 'na')"#,
            now,
            ACCOUNT_ID
        )
        .execute(&pool)
        .await
        .expect("db setup");

        let now = now + chrono::Duration::seconds(1);
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'complained', 'na')"#,
            now,
            ACCOUNT_ID
        )
        .execute(&pool)
        .await
        .expect("db setup");

        let now = now + chrono::Duration::seconds(1);
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'complained', 'na')"#,
            now,
            ACCOUNT_ID
        )
        .execute(&pool)
        .await
        .expect("db setup");

        // Add one delivered email
        let now = chrono::Utc::now();
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
        VALUES ($1, $2, 'delivered', 'na')"#,
            now,
            ACCOUNT_ID
        )
        .execute(&pool)
        .await
        .expect("db setup");

        pool
    }

    async fn sent_emails(pool: &mut SqlitePool) -> i32 {
        sqlx::query_scalar!(
            r#"SELECT
                COUNT(*) AS sent
            FROM emails e
            WHERE e.account_id = $1
                AND e.state = 'sent'"#,
            ACCOUNT_ID
        )
        .fetch_one(&*pool)
        .await
        .expect("db setup")
    }

    async fn context() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("db setup");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("db setup");
        sqlx::query!(
            r#"INSERT INTO users (id, email)
            VALUES ($1, $2)"#,
            ACCOUNT_ID,
            USER_EMAIL
        )
        .execute(&pool)
        .await
        .expect("db setup");

        sqlx::query!(
            r#"INSERT INTO accounts (id, userId, type, provider, providerAccountId)
            VALUES ($1, $1, 'email', 'email', $2)"#,
            ACCOUNT_ID,
            USER_EMAIL
        )
        .execute(&pool)
        .await
        .expect("db setup");

        pool
    }
}
