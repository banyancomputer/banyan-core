use crate::database::DatabaseConnection;

pub async fn created(conn: &mut DatabaseConnection, session: &stripe::Invoice) -> Result<(), sqlx::Error> {
    todo!()
}

pub async fn finalization_failed(conn: &mut DatabaseConnection, session: &stripe::Invoice) -> Result<(), sqlx::Error> {
    todo!()
}

pub async fn finalized(conn: &mut DatabaseConnection, session: &stripe::Invoice) -> Result<(), sqlx::Error> {
    todo!()
}

pub async fn paid(conn: &mut DatabaseConnection, session: &stripe::Invoice) -> Result<(), sqlx::Error> {
    todo!()
}

pub async fn payment_action_required(conn: &mut DatabaseConnection, session: &stripe::Invoice) -> Result<(), sqlx::Error> {
    todo!()
}

pub async fn payment_failed(conn: &mut DatabaseConnection, session: &stripe::Invoice) -> Result<(), sqlx::Error> {
    todo!()
}

pub async fn updated(conn: &mut DatabaseConnection, session: &stripe::Invoice) -> Result<(), sqlx::Error> {
    todo!()
}
