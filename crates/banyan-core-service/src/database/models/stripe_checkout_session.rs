use time::OffsetDateTime;

use crate::database::models::StripeCheckoutSessionStatus;
use crate::database::DatabaseConnection;

pub struct NewStripeCheckoutSession<'a> {
    pub user_id: &'a str,
    pub session_id: &'a str,
    pub stripe_checkout_session_id: &'a str,
}

impl<'a> NewStripeCheckoutSession<'a> {
    pub async fn save(self, conn: &mut DatabaseConnection) -> Result<String, sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        sqlx::query_scalar!(
            r#"INSERT INTO stripe_checkout_sessions (user_id, session_id, stripe_checkout_session_id, status, created_at)
                 VALUES ($1, $2, $3, $4, $5)
                 RETURNING id;"#,
            self.user_id,
            self.session_id,
            self.stripe_checkout_session_id,
            StripeCheckoutSessionStatus::Created,
            now,
        )
        .fetch_one(&mut *conn)
        .await
    }
}

#[derive(sqlx::FromRow)]
pub struct StripeCheckoutSession {
    pub id: String,

    pub user_id: String,
    pub session_id: String,

    pub stripe_checkout_session_id: String,
    pub status: StripeCheckoutSessionStatus,

    pub created_at: OffsetDateTime,
    pub completed_at: Option<OffsetDateTime>,
}

impl StripeCheckoutSession {
    pub async fn find_by_id(
        conn: &mut DatabaseConnection,
        id: &str,
    ) -> Result<Option<StripeCheckoutSession>, sqlx::Error> {
        sqlx::query_as!(
            StripeCheckoutSession,
            r#"SELECT id, user_id, session_id, stripe_checkout_session_id, status as 'status: StripeCheckoutSessionStatus',
                   created_at, completed_at FROM stripe_checkout_sessions
                 WHERE id = $1;"#,
            id,
        )
        .fetch_optional(&mut *conn)
        .await
    }
}
