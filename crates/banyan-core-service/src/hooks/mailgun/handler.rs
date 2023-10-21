use axum::extract::{self, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::Connection;

use super::request::MailgunHookRequest;

use crate::db::models::EmailMessageState;
use crate::error::CoreError;
use crate::extractors::{DbConn, MailgunSigningKey};

pub async fn handle(
    mut db_conn: DbConn,
    mailgun_signing_key: MailgunSigningKey,
    Json(hook_request): extract::Json<MailgunHookRequest>,
) -> Response {

    // ...


    let email_stat_query = format!(
        "INSERT INTO email_stats(account_id, {}) VALUES ($1, 1) ON CONFLICT(account_id) DO UPDATE SET {} = {} + 1 WHERE account_id = $1;",
        next_state, next_state, next_state
    );

    // No need to do anything to the state but we should update the email_stat counter
    if next_state < current_state {
        match sqlx::query(&email_stat_query)
            .bind(account_id)
            .execute(&mut *db_conn.0)
            .await
        {
            Ok(_) => return (StatusCode::OK).into_response(),
            Err(err) => {
                return CoreError::generic_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "backend service issue",
                    Some(&format!("failed to update email_stats: {err}")),
                )
                .into_response()
            }
        }
    }

    let next_state = next_state.to_string();
    // Otherwise, this is a valid state transition. Start a transaction and update the email state
    let mut tx = match db_conn.0.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            return CoreError::generic_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "backend service issue",
                Some(&format!("failed to start transaction: {err}")),
            )
            .into_response()
        }
    };

    match sqlx::query!(
        r#"UPDATE emails SET state = $1 WHERE id = $2;"#,
        next_state,
        message_id
    )
    .execute(&mut *tx)
    .await
    {
        Ok(_) => (),
        Err(err) => {
            return CoreError::generic_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "backend service issue",
                Some(&format!("failed to update email state: {err}")),
            )
            .into_response()
        }
    }

    match sqlx::query(&email_stat_query)
        .bind(account_id)
        .execute(&mut *tx)
        .await
    {
        Ok(_) => (),
        Err(err) => {
            return CoreError::generic_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "backend service issue",
                Some(&format!("failed to update email_stats: {err}")),
            )
            .into_response()
        }
    }

    match tx.commit().await {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(err) => CoreError::generic_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "backend service issue",
            Some(&format!("failed to commit transaction: {err}")),
        )
        .into_response(),
    }
}
