use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::models::ApiBucket;
use crate::app::AppState;
use crate::database::models::Bucket;
use crate::extractors::UserIdentity;

pub async fn handler(user_identity: UserIdentity, State(state): State<AppState>) -> Response {
    let database = state.database();

    let user_id = user_identity.id().to_string();
    let query_result = sqlx::query_as!(
        Bucket,
        "SELECT * FROM buckets WHERE user_id = $1;",
        user_id,
    )
    .fetch_all(&database)
    .await;

    // note: this also includes user_id which wasn't being returned before and may cause
    // compatibility issues

    match query_result {
        Ok(qr) => {
            let buckets: Vec<_> = qr.into_iter().map(ApiBucket::from).collect();
            (StatusCode::OK, Json(buckets)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to lookup all buckets for account: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
