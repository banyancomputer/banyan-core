use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::database::models::{Bucket, BucketType, StorageClass};
use crate::extractors::ApiToken;

use crate::app::AppState;

pub async fn handler(api_token: ApiToken, State(state): State<AppState>) -> Response {
    let database = state.database();

    let query_result = sqlx::query_as!(
        Bucket,
        "SELECT * FROM buckets WHERE account_id = $1;",
        api_token.subject,
    )
    .fetch_all(&database)
    .await;

    // note: this also includes account_id which wasn't being returned before and may cause
    // compatibility issues

    match query_result {
        Ok(qr) => {
            let buckets: Vec<_> = qr.into_iter().map(|db| ApiBucket::from(db)).collect();
            (StatusCode::OK, Json(buckets)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to lookup all buckets for account: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}

#[derive(Serialize)]
pub struct ApiBucket {
    pub id: String,
    pub name: String,
    pub r#type: BucketType,
    pub storage_class: StorageClass,
}

impl From<Bucket> for ApiBucket {
    fn from(value: Bucket) -> Self {
        Self {
            id: value.id,
            name: value.name,
            r#type: value.r#type,
            storage_class: value.storage_class,
        }
    }
}
