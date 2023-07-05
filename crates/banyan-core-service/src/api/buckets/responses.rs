use axum::Json;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;
use uuid::Uuid;

use crate::api::buckets::BucketError;
use crate::util::collect_error_messages;

#[derive(Serialize)]
pub struct Bucket {
    pub uuid: Uuid,
    pub friendly_name: String,
    pub r#type: BucketType,

    pub meta_data_cid: String,
    pub public_keys: Vec<PublicKey>,
}

impl IntoResponse for Bucket {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BucketType {
    Backup,
    Interactive,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Client {
    Web,
    Api {
        friendly_name: String,
        id: Uuid,
    },
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub errors: Vec<String>,
}

impl From<BucketError> for ErrorResponse {
    fn from(value: BucketError) -> Self {
        Self {
            errors: collect_error_messages(value),
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}

#[derive(Serialize)]
pub struct ProtectedKey(pub String);

#[derive(Serialize)]
pub struct PublicKey {
    pub client: Client,
    pub fingerprint: String,
    pub status: PublicKeyStatus,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicKeyStatus {
    Approved(ProtectedKey),
    Pending,
}
