use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Serialize, Serializer};
use serde::ser::SerializeMap;

use crate::utils::db;
use crate::extractors::{ApiToken, DbConn};

/// Currently all users have a capacity threshold of 5TiB
const STATIC_USAGE_THRESHOLD: u64 = 5 * 1024_u64.pow(4);

pub async fn handler(api_token: ApiToken, mut db_conn: DbConn) -> Result<Response, AccountInfoError> {
    let account_info = get_account_info(&mut db_conn, &api_token.subject()).await?;
    Ok((StatusCode::OK, Json(account_info)).into_response())
}

// todo: change account_id type to Uuid
async fn get_account_info(db_conn: &mut DbConn, account_id: &str) -> Result<AccountInfo, AccountInfoError> {
    let current_usage = match db::read_total_data_usage(account_id, db_conn).await {
        Ok(usage) => usage,
        Err(err) if matches!(err, sqlx::Error::RowNotFound) => return Err(AccountInfoError::NotFound),
        Err(err) => {
            tracing::error!("failed to retrieve account info from database: {err:?}");
            return Err(AccountInfoError::DatabaseIssue);
        },
    };

    Ok(AccountInfo {
        current_usage,
        usage_limit: STATIC_USAGE_THRESHOLD,
    })
}

#[derive(Debug, Serialize)]
struct AccountInfo {
    current_usage: u64,
    usage_limit: u64,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AccountInfoError {
    #[error("unable to query for current account info")]
    DatabaseIssue,

    #[error("account not found")]
    NotFound,
}

impl IntoResponse for AccountInfoError {
    fn into_response(self) -> Response {
        use AccountInfoError::*;

        let status_code = match self {
            DatabaseIssue => StatusCode::INTERNAL_SERVER_ERROR,
            NotFound => StatusCode::NOT_FOUND,
        };

        (status_code, Json(self)).into_response()
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
