use crate::database::DatabaseConnection;

pub struct NewStripeCheckoutSession<'a> {
    pub user_id: &'a str,
    pub session_id: &'a str,
    pub stripe_checkout_session_id: &'a str,
}

impl<'a> NewStripeCheckoutSession<'a> {
    pub async fn save(self, conn: &mut DatabaseConnection) -> Result<String, sqlx::Error> {
        todo!()
    }
}

pub struct StripeCheckoutSession;

impl StripeCheckoutSession {
    pub async fn complete(conn: &mut DatabaseConnection, session_id: &str, id: &str) -> Result<(), sqlx::Error> {
        todo!()
    }
}
