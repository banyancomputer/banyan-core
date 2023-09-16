use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use serde::{Serialize, Serializer};
use serde::ser::SerializeMap;

use crate::app_state::AppState;
use crate::extractors::{ApiToken, DbConn};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/info", get(account_info))
        .with_state(state)
}

async fn account_info(api_token: ApiToken, mut db_conn: DbConn) -> Result<Response, AccountInfoError> {
    let account_info = get_account_info(&mut db_conn, &api_token.subject())?;
    Ok((StatusCode::OK, Json(account_info)).into_response())
}

// todo: change account_id type to Uuid
fn get_account_info(_db_conn: &mut DbConn, _account_id: &str) -> Result<AccountInfo, AccountInfoError> {
    todo!()
}

#[derive(Debug, Serialize)]
struct AccountInfo {
    current_usage: usize,
    usage_limit: usize,
}

#[derive(Debug, thiserror::Error)]
enum AccountInfoError {
    #[error("account not found")]
    NotFound,
}

impl IntoResponse for AccountInfoError {
    fn into_response(self) -> Response {
        use AccountInfoError::*;

        match self {
            NotFound => (StatusCode::NOT_FOUND, Json(self)).into_response(),
        }
    }
}

impl Serialize for AccountInfoError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("msg", &self.to_string())?;
        map.end()
    }
}
