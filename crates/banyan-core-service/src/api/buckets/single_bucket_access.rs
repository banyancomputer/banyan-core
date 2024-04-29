use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiBucketAccess;
use crate::app::AppState;
use crate::database::models::BucketAccessState;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path((bucket_id, user_key_id)): Path<(Uuid, Uuid)>,
) -> Response {
    let database = state.database();

    let bucket_id = bucket_id.to_string();
    let user_key_id = user_key_id.to_string();

    let user_id = user_identity.id().to_string();
    let query_result = sqlx::query_as!(
        ApiBucketAccess,
        r#"
            SELECT ba.user_key_id, ba.bucket_id, uk.fingerprint, ba.state AS 'state: BucketAccessState'
            FROM bucket_access AS ba
            JOIN user_keys AS uk ON uk.id = ba.user_key_id
            JOIN users AS u on u.id = uk.user_id
            JOIN buckets AS b on b.id = ba.bucket_id
            WHERE uk.id = $1
            AND u.id = $2
            AND b.id = $3;
        ;"#,
        user_key_id,
        user_id,
        bucket_id,
    )
    .fetch_one(&database)
    .await;

    match query_result {
        Ok(ba) => (StatusCode::OK, Json(ba)).into_response(),
        Err(sqlx::Error::RowNotFound) => {
            let err_msg = serde_json::json!({"msg": "not found"});
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to lookup specific bucket access for account: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
