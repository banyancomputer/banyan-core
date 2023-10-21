#![allow(dead_code)]

use uuid::Uuid;

use crate::database::models::{BucketType, EmailMessageState, StorageClass};
use crate::database::Database;
use crate::email::message::EmailMessage;

/// Create a new Bucket in the database and return the created resource. Implements an authorized
/// read of a bucket by id and account_id.
pub async fn create_bucket(
    account_id: &str,
    name: &str,
    r#type: &BucketType,
    storage_class: &StorageClass,
    database: &Database,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(
        r#"INSERT INTO buckets (account_id, name, type, storage_class)
               VALUES ($1, $2, $3, $4)
               RETURNING id;"#,
        account_id,
        name,
        r#type,
        storage_class,
    )
    .fetch_one(database)
    .await
}

/// Record a new email in the database as queued for sending.
pub async fn queue_email(
    account_id: &str,
    email_message: &impl EmailMessage,
    database: &Database,
) -> Result<Uuid, sqlx::Error> {
    let type_name = email_message.type_name();

    let email_id: String = sqlx::query_scalar!(
        r#"INSERT INTO emails (account_id, type) VALUES ($1, $2) RETURNING id;"#,
        account_id,
        type_name
    )
    .fetch_one(database)
    .await?;

    // todo: remove expect here
    let message_id = Uuid::parse_str(&email_id).expect("queue_email: invalid uuid");

    Ok(message_id)
}

/// Set en email as sent
pub async fn send_email(message_id: Uuid, database: &Database) -> Result<(), sqlx::Error> {
    let message_id = message_id.to_string();

    sqlx::query!(
        r#"UPDATE emails
               SET state = 'sent',
                   sent_at = CURRENT_TIMESTAMP
               WHERE id = $1;"#,
        message_id
    )
    .execute(database)
    .await?;

    Ok(())
}

/// Read the current state of an email in the database.
pub async fn read_email_state(
    message_id: Uuid,
    database: &Database,
) -> Result<EmailMessageState, sqlx::Error> {
    let message_id = message_id.to_string();

    sqlx::query_scalar!(
        r#"SELECT state as 'state: EmailMessageState'
               FROM emails
               WHERE id = $1;"#,
        message_id,
    )
    .fetch_one(database)
    .await
}

/// Update the state of an email in the database.
pub async fn update_email_state(
    message_id: Uuid,
    state: EmailMessageState,
    database: &Database,
) -> Result<(), sqlx::Error> {
    let message_id = message_id.to_string();
    let state = state.to_string();

    sqlx::query!(
        r#"UPDATE emails
               SET state = $1
               WHERE id = $2;"#,
        state,
        message_id
    )
    .execute(database)
    .await?;

    Ok(())
}
