use crate::database::DatabaseConnection;

pub async fn created(conn: &mut DatabaseConnection, session: &stripe::PaymentIntent) -> Result<(), sqlx::Error> {
    todo!()
}

pub async fn succeeded(conn: &mut DatabaseConnection, session: &stripe::PaymentIntent) -> Result<(), sqlx::Error> {
    todo!()
}
