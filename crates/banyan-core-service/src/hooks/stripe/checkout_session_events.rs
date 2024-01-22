use crate::database::DatabaseConnection;

pub async fn completed(conn: &mut DatabaseConnection, session: &stripe::CheckoutSession) -> Result<(), sqlx::Error> {
    todo!()
}

pub async fn expired(conn: &mut DatabaseConnection, session: &stripe::CheckoutSession) -> Result<(), sqlx::Error> {
    todo!()
}
