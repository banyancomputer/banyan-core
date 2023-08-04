use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::extractors::ApiToken;

pub async fn destroy(
    _api_token: ApiToken,
    Path(_bucket_id): Path<Uuid>,
    Path(_metadata_id): Path<Uuid>,
) -> Response {
    (StatusCode::NO_CONTENT, ()).into_response()
}

pub async fn download(
    _api_token: ApiToken,
    Path(_bucket_id): Path<Uuid>,
    Path(_metadata_id): Path<Uuid>,
) -> Response {
    (StatusCode::NOT_FOUND, ()).into_response()
}

pub async fn index(_api_token: ApiToken, Path(_bucket_id): Path<Uuid>) -> Response {
    let response = serde_json::json!([
        { "id": "e627f0cc-1cfc-42fb-a8cb-23a57edc4594", "metadata_size": 1_187, "state": "pending" },
        { "id": "8d834721-c707-41cb-937e-ccbf5eb2102a", "metadata_size": 41_378, "state": "current" },
        { "id": "4b35955f-8a10-4b97-b9d3-857fde03106a", "metadata_size": 41_378, "state": "snapshot" },
    ]);

    (StatusCode::OK, axum::Json(response)).into_response()
}

pub async fn show(
    _api_token: ApiToken,
    Path(_bucket_id): Path<Uuid>,
    Path(metadata_id): Path<Uuid>,
) -> Response {
    let response = serde_json::json!({
        "id": metadata_id,
        "state": "pending",

        "data_size": 1_567_120,
        "metadata_size": 78_120,

        "published_at": "20230804T171200+Z",

        "authorized_public_keys": [
            "98:01:73:9d:aa:e4:4e:c5:29:3d:4e:1f:53:d3:f4:d2:d4:26:d9:1c",
        ],
        "storage_providers": [
            "http://127.0.0.1:3002",
        ],
    });

    (StatusCode::OK, axum::Json(response)).into_response()
}

pub async fn snapshot(
    _api_token: ApiToken,
    Path(_bucket_id): Path<Uuid>,
    Path(_metadata_id): Path<Uuid>,
) -> Response {
    (StatusCode::UNAUTHORIZED, ()).into_response()
}
