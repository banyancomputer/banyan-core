use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::AppState;
use crate::extractors::ApiIdentity;

#[derive(sqlx::FromRow)]
struct ConsumedStorage {
    data_size: i32,
    meta_size: i32,
}

pub async fn handler(api_id: ApiIdentity, State(state): State<AppState>) -> Response {
    let database = state.database();

    // we need to include outdated currently as they include blocks referenced by the current
    // version, todo: we'll need a better way of calculating this
    let query_result = sqlx::query_as!(
        ConsumedStorage,
        r#"SELECT
            COALESCE(SUM(m.metadata_size), 0) as data_size,
            COALESCE(SUM(COALESCE(m.data_size, m.expected_data_size)), 0) as meta_size
        FROM
            metadata m
        INNER JOIN
            buckets b ON b.id = m.bucket_id
        WHERE
            b.account_id = $1 AND m.state IN ('current', 'outdated', 'pending');"#,
        api_id.account_id,
    )
    .fetch_one(&database)
    .await;

    match query_result {
        Ok(store) => {
            let resp = serde_json::json!({"size": store.data_size + store.meta_size});
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to calculate current total usage: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
