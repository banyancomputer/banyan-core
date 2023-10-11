use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::app::AppState;
use crate::extractors::ApiToken;

pub async fn handler(api_token: ApiToken, State(state): State<AppState>) -> Response {
    let account_id = api_token.subject();
    let database = state.database();

    // we need to include outdated currently as they include blocks referenced by the current
    // version, todo: we'll need a better way of calculating this
    let query_result: Result<(i64, i64), _> = sqlx::query_as(
        r#"SELECT
            SUM(COALESCE(COALESCE(m.data_size, m.expected_data_size), 0)) as data_size,
            SUM(COALESCE(m.metadata_size, 0)) as metadata_size
        FROM
            metadata m
        INNER JOIN
            buckets b ON b.id = m.bucket_id
        WHERE
            b.account_id = $1 AND m.state IN ('current', 'outdated', 'pending');"#
    )
    .bind(account_id)
    .fetch_one(&database)
    .await;

    match query_result {
        Ok((data_size, metadata_size)) => {
            let resp = serde_json::json!({"size": data_size + metadata_size});
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to calculate current total usage: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
