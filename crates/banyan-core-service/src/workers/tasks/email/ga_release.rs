#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::email::message::GaRelease;
use crate::workers::CurrentTask;
use crate::workers::TaskLike;

use super::send_email_message;
use super::should_send_email_message;
use super::EmailTaskContext;
use super::EmailTaskError;

#[derive(Deserialize, Serialize)]
pub struct GaReleaseEmailTask {
    account_id: Uuid,
}

impl GaReleaseEmailTask {
    pub fn new(account_id: Uuid) -> Self {
        Self { account_id }
    }
}

#[async_trait]
impl TaskLike for GaReleaseEmailTask {
    const TASK_NAME: &'static str = "ga_release_email_task";

    type Error = EmailTaskError;
    type Context = EmailTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        // Filter out innapropriate emails
        if !should_send_email_message(self.account_id, &ctx).await? {
            return Ok(());
        }
        let message = GaRelease {};
        send_email_message(self.account_id, &message, &ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::email::config::EmailConfig;
    use crate::workers::current_task::tests::default_current_task;
    use sqlx::SqlitePool;

    const ACCOUNT_ID: &str = "00000000-0000-0000-0000-000000000000";
    const USER_EMAIL: &str = "user@user.email";

    #[tokio::test]
    async fn success() -> Result<(), EmailTaskError> {
        let ctx = context().await;
        email_task(ctx.clone()).await?;
        let email_count = count_sent_emails(&ctx).await;
        assert_eq!(email_count, 1);
        Ok(())
    }

    #[tokio::test]
    async fn unsubscribed() -> Result<(), EmailTaskError> {
        let ctx = context().await;
        // Create one unsubscribed email as the last email sent
        let now = chrono::Utc::now();
        let _later = unsubscribed_email(&ctx, now).await;
        email_task(ctx.clone()).await?;
        let email_count = count_sent_emails(&ctx).await;
        assert_eq!(email_count, 0);
        Ok(())
    }

    #[tokio::test]
    async fn unsubscribed_then_sent() -> Result<(), EmailTaskError> {
        let ctx = context().await;
        // One unsubscribed email then one delivered email as the last email sent
        let now = chrono::Utc::now();
        let later = unsubscribed_email(&ctx, now).await;
        let _later = delivered_email(&ctx, later).await;
        email_task(ctx.clone()).await?;
        let email_count = count_sent_emails(&ctx).await;
        assert_eq!(email_count, 1);
        Ok(())
    }

    #[tokio::test]
    async fn failure() -> Result<(), EmailTaskError> {
        let ctx = context().await;
        // Threee failed emails in a row as the last three emails sent
        let now = chrono::Utc::now();
        let later = failed_email(&ctx, now).await;
        let later = failed_email(&ctx, later).await;
        let _later = failed_email(&ctx, later).await;
        email_task(ctx.clone()).await?;
        let email_count = count_sent_emails(&ctx).await;
        assert_eq!(email_count, 0);
        Ok(())
    }

    #[tokio::test]
    async fn failure_then_sent() -> Result<(), EmailTaskError> {
        let ctx = context().await;
        // Threee failed emails in a row and then one delivered email
        let now = chrono::Utc::now();
        let later = failed_email(&ctx, now).await;
        let later = failed_email(&ctx, later).await;
        let later = failed_email(&ctx, later).await;
        let _later = delivered_email(&ctx, later).await;
        email_task(ctx.clone()).await?;
        let email_count = count_sent_emails(&ctx).await;
        assert_eq!(email_count, 1);
        Ok(())
    }

    #[tokio::test]
    async fn complaint() -> Result<(), EmailTaskError> {
        let ctx = context().await;
        let now = chrono::Utc::now();
        // Three complaints interspersed with delivered emails
        let later = complained_email(&ctx, now).await;
        let later = delivered_email(&ctx, later).await;
        let later = complained_email(&ctx, later).await;
        let later = delivered_email(&ctx, later).await;
        let _later = complained_email(&ctx, later).await;
        email_task(ctx.clone()).await?;
        let email_count = count_sent_emails(&ctx).await;
        assert_eq!(email_count, 0);
        Ok(())
    }

    async fn email_task(ctx: EmailTaskContext) -> Result<(), EmailTaskError> {
        let task = GaReleaseEmailTask::new(Uuid::parse_str(ACCOUNT_ID).unwrap());
        task.run(default_current_task(), ctx).await?;
        Ok(())
    }

    async fn count_sent_emails(ctx: &EmailTaskContext) -> i32 {
        let mut db_conn = ctx.db_pool().acquire().await.unwrap();
        sqlx::query_scalar!(
            r#"SELECT
                COUNT(*) AS sent
            FROM emails e
            WHERE e.account_id = $1
                AND e.state = 'sent'"#,
            ACCOUNT_ID
        )
        .fetch_one(&mut *db_conn)
        .await
        .expect("db setup")
    }

    async fn delivered_email(
        ctx: &EmailTaskContext,
        when: chrono::DateTime<chrono::Utc>,
    ) -> chrono::DateTime<chrono::Utc> {
        let mut db_conn = ctx.db_pool().acquire().await.unwrap();
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'delivered', 'na')"#,
            when,
            ACCOUNT_ID
        )
        .execute(&mut *db_conn)
        .await
        .expect("db setup");
        when + chrono::Duration::seconds(1)
    }

    async fn failed_email(
        ctx: &EmailTaskContext,
        when: chrono::DateTime<chrono::Utc>,
    ) -> chrono::DateTime<chrono::Utc> {
        let mut db_conn = ctx.db_pool().acquire().await.unwrap();
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'failed', 'na')"#,
            when,
            ACCOUNT_ID
        )
        .execute(&mut *db_conn)
        .await
        .expect("db setup");
        when + chrono::Duration::seconds(1)
    }

    async fn complained_email(
        ctx: &EmailTaskContext,
        when: chrono::DateTime<chrono::Utc>,
    ) -> chrono::DateTime<chrono::Utc> {
        let mut db_conn = ctx.db_pool().acquire().await.unwrap();
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'complained', 'na')"#,
            when,
            ACCOUNT_ID
        )
        .execute(&mut *db_conn)
        .await
        .expect("db setup");
        when + chrono::Duration::seconds(1)
    }

    async fn unsubscribed_email(
        ctx: &EmailTaskContext,
        when: chrono::DateTime<chrono::Utc>,
    ) -> chrono::DateTime<chrono::Utc> {
        let mut db_conn = ctx.db_pool().acquire().await.unwrap();
        sqlx::query!(
            r#"INSERT INTO emails (sent_at, account_id, state, type)
            VALUES ($1, $2, 'unsubscribed', 'na')"#,
            when,
            ACCOUNT_ID
        )
        .execute(&mut *db_conn)
        .await
        .expect("db setup");
        when + chrono::Duration::seconds(1)
    }

    async fn context() -> EmailTaskContext {
        let db_conn = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("db setup");
        sqlx::migrate!("./migrations")
            .run(&db_conn)
            .await
            .expect("db setup");
        sqlx::query!(
            r#"INSERT INTO users (id, email)
            VALUES ($1, $2)"#,
            ACCOUNT_ID,
            USER_EMAIL
        )
        .execute(&db_conn)
        .await
        .expect("db setup");

        sqlx::query!(
            r#"INSERT INTO accounts (id, userId, type, provider, providerAccountId)
            VALUES ($1, $1, 'email', 'email', $2)"#,
            ACCOUNT_ID,
            USER_EMAIL
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
