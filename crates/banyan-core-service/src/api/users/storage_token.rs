use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::models::ApiUser;
use crate::app::{AppState, ServiceKey};
use crate::database::models::{MetricsTraffic, User};
use crate::database::Database;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    database: Database,
    service_key: ServiceKey,
    Path((domain, port)): Path<(String, Option<u16>)>,
) -> Response {
    let port = port.unwrap_or(443);

    todo!()
}
