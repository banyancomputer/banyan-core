use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use time::OffsetDateTime;

use crate::app::AppState;
use crate::database::models::{User, TaxClass};
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Json(update_request): Json<UpdateApiUserRequest>,
) -> Result<Response, UpdateUserError> {
    let database = state.database();
    let mut conn = database.begin().await?;

    let user_id = user_identity.id().to_string();
    let user: User = User::find_by_id(&mut *conn, &user_id).await?.ok_or(UpdateUserError::NotFound)?;

    // If these aren't provided use our existing values for the user
    let account_tax_class = update_request.account_tax_class.unwrap_or(user.account_tax_class);
    let accepted_tos_at = match update_request.accepted_tos_at {
        Some(tos) => {
            let tos = OffsetDateTime::from_unix_timestamp(tos).map_err(|_| UpdateUserError::InvalidTimestamp)?;

            // Going backwards on this value is an error
            if let Some(existing_tos) = user.accepted_tos_at {
                if tos < existing_tos {
                    return Err(UpdateUserError::InvalidTimestamp);
                }
            }

            Some(tos)
        }
        None => user.accepted_tos_at,
    };

    sqlx::query!(
        r#"UPDATE users SET accepted_tos_at = $1 AND account_tax_class = $2 WHERE id = $3 AND (accepted_tos_at != $1 OR account_tax_class != $2);"#,
        accepted_tos_at,
        account_tax_class,
        user_id,
    )
    .execute(&mut *conn)
    .await?;

    conn.commit().await?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Deserialize)]
pub struct UpdateApiUserRequest {
    accepted_tos_at: Option<i64>,

    account_tax_class: Option<TaxClass>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateUserError {
    #[error("database query failed: {0}")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("acceptance of terms of service can not be reverted and must be a valid unix timestamp")]
    InvalidTimestamp,

    #[error("user does not exist")]
    NotFound,
}

impl IntoResponse for UpdateUserError {
    fn into_response(self) -> Response {
        match self {
            UpdateUserError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            UpdateUserError::InvalidTimestamp => {
                let err_msg = serde_json::json!({"msg": self.to_string()});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("encountered error reading user: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
