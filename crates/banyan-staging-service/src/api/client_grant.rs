use std::str::FromStr;

use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::State as AppState;
use crate::database::{DbError, DbResult, Executor};
use crate::extractors::{Database, StorageGrant};

#[derive(Deserialize, Serialize)]
pub struct GrantRequest {
    public_key: String,
}

#[axum::debug_handler]
pub async fn handler(
    // this weirdly needs to be present even though we don't use it
    State(_state): State<AppState>,
    database: Database,
    grant: StorageGrant,
    Json(request): Json<GrantRequest>,
) -> Response {
    let maybe_existing_user = existing_grant_user(&database, &grant).await.map_err(|err| return GrantError::Database(err).into_response());
    //let grant_user_id = match 
    //    Ok(Some(uuid)) => uuid,

    //};

    let msg = serde_json::json!({"msg": "success"});
    (StatusCode::NO_CONTENT, axum::Json(msg)).into_response()
}

use sqlx::FromRow;

#[derive(FromRow)]
struct BareId {
    id: String,
}

async fn existing_grant_user(database: &Database, grant: &StorageGrant) -> DbResult<Option<Uuid>> {
    match database.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            let user_id: Option<BareId> = sqlx::query_as("SELECT id FROM clients WHERE fingerprint = $1")
                .bind(grant.client_fingerprint())
                .fetch_optional(conn)
                .await
                .map_err(postgres::map_sqlx_error)?;

            Ok(user_id.map(|b| Uuid::parse_str(b.id.as_str()).unwrap()))
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

            let user_id: Option<BareId> = sqlx::query_as("SELECT id FROM clients WHERE fingerprint = $1")
                .bind(grant.client_fingerprint())
                .fetch_optional(conn)
                .await
                .map_err(sqlite::map_sqlx_error)?;

            Ok(user_id.map(|b| Uuid::parse_str(b.id.as_str()).unwrap()))
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum GrantError {
    #[error("database issue occurred")]
    Database(#[from] DbError),
}

impl IntoResponse for GrantError {
    fn into_response(self) -> Response {
        use GrantError::*;

        match &self {
            Database(err) => {
                tracing::error!(error = ?err, "a database error occurred");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
