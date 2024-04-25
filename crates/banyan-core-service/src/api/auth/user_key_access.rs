use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::api::models::ApiUserKeyAccess;
use crate::app::AppState;
use crate::database::models::UserKeyAccess;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
) -> Result<Response, UserKeyAccessError> {
    let database = state.database();
    let user_id = user_identity.id().to_string();

    let user_key_access_states: Vec<UserKeyAccess> = sqlx::query_as!(
        UserKeyAccess,
        r#"
            SELECT 
                uk.id, 
                uk.user_id, 
                uk.pem, 
                uk.fingerprint, 
                GROUP_CONCAT(b.id) AS "bucket_ids!: String"
            FROM user_keys AS uk
            LEFT OUTER JOIN bucket_access AS ba ON ba.user_key_id = uk.id
            LEFT OUTER JOIN buckets AS b ON b.id = ba.bucket_id
            WHERE (
                ba.user_key_id IS NULL
                AND
                uk.user_id = $1
            ) OR (
                b.id IN (
                    SELECT b2.id FROM buckets AS b2
                    JOIN bucket_access AS ba2 ON ba2.bucket_id = b2.id
                    JOIN user_keys AS uk2 ON uk2.id = ba2.user_key_id
                    WHERE uk2.user_id = $1
                )
            )
            GROUP BY uk.id;
        "#,
        user_id,
    )
    .fetch_all(&database)
    .await?;

    Ok((
        StatusCode::OK,
        Json(
            user_key_access_states
                .into_iter()
                .map(Into::<ApiUserKeyAccess>::into)
                .collect::<Vec<_>>(),
        ),
    )
        .into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum UserKeyAccessError {
    #[error("database query failures: {0}")]
    DatabaseFailure(#[from] sqlx::Error),
}

impl IntoResponse for UserKeyAccessError {
    fn into_response(self) -> Response {
        match &self {
            _ => {
                tracing::error!("a stripe webhook error occurred: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
