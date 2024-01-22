use crate::database::DatabaseConnection;

pub async fn created(conn: &mut DatabaseConnection, session: &stripe::Subscription) -> Result<(), sqlx::Error> {
    todo!()
}

pub async fn deleted(conn: &mut DatabaseConnection, session: &stripe::Subscription) -> Result<(), sqlx::Error> {
    todo!()
}

pub async fn paused(conn: &mut DatabaseConnection, session: &stripe::Subscription) -> Result<(), sqlx::Error> {
    todo!()
}

pub async fn resumed(conn: &mut DatabaseConnection, session: &stripe::Subscription) -> Result<(), sqlx::Error> {
    todo!()
}

pub async fn updated(conn: &mut DatabaseConnection, session: &stripe::Subscription) -> Result<(), sqlx::Error> {
    todo!()
}
