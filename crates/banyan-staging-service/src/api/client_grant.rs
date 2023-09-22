use std::str::FromStr;

use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::State as AppState;
use crate::database::{BareId, DbError, DbResult, Executor};
use crate::extractors::{Database, StorageGrant};

#[derive(Deserialize, Serialize)]
pub struct GrantRequest {
    public_key: String,
}

pub async fn handler(
    // this weirdly needs to be present even though we don't use it
    _: State<AppState>,
    database: Database,
    grant: StorageGrant,
    Json(request): Json<GrantRequest>,
) -> Result<Response, GrantError> {
    let grant_user_id = ensure_grant_user(&database, &grant, request).await?;
    create_storage_grant(grant_user_id, &database, &grant).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

async fn ensure_grant_user(
    database: &Database,
    grant: &StorageGrant,
    request: GrantRequest,
) -> Result<Uuid, GrantError> {
    match existing_grant_user(database, grant).await? {
        Some(uuid) => Ok(uuid),
        None => create_grant_user(database, grant, request).await,
    }
}

async fn create_grant_user(
    database: &Database,
    grant: &StorageGrant,
    request: GrantRequest,
) -> Result<Uuid, GrantError> {
    match database.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            let client_id : String = sqlx::query_scalar("INSERT INTO clients (platform_id, fingerprint, public_key) VALUES ($1::uuid, $2, $3) RETURNING CAST(id AS TEXT) as id;")
                .bind(grant.platform_id().to_string())
                .bind(grant.client_fingerprint())
                .bind(request.public_key)
                .fetch_one(conn)
                .await
                .map_err(postgres::map_sqlx_error)
                .map_err(GrantError::Database)?;

            Ok(Uuid::parse_str(&client_id).unwrap())
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

            let client_id: String = sqlx::query_scalar("INSERT INTO clients (platform_id, fingerprint, public_key) VALUES ($1, $2, $3) RETURNING id;")
                .bind(grant.platform_id().to_string())
                .bind(grant.client_fingerprint())
                .bind(request.public_key)
                .fetch_one(conn)
                .await
                .map_err(sqlite::map_sqlx_error)
                .map_err(GrantError::Database)?;

            Ok(Uuid::parse_str(&client_id).unwrap())
        }
    }
}

async fn existing_grant_user(
    database: &Database,
    grant: &StorageGrant,
) -> Result<Option<Uuid>, GrantError> {
    match database.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            let user_id: Option<BareId> =
                sqlx::query_as("SELECT CAST(id AS TEXT) as id FROM clients WHERE fingerprint = $1")
                    .bind(grant.client_fingerprint())
                    .fetch_optional(conn)
                    .await
                    .map_err(postgres::map_sqlx_error)
                    .map_err(GrantError::Database)?;

            Ok(user_id.map(|b| Uuid::parse_str(b.id.as_str()).unwrap()))
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

            let user_id: Option<BareId> =
                sqlx::query_as("SELECT id FROM clients WHERE fingerprint = $1")
                    .bind(grant.client_fingerprint())
                    .fetch_optional(conn)
                    .await
                    .map_err(sqlite::map_sqlx_error)
                    .map_err(GrantError::Database)?;

            Ok(user_id.map(|b| Uuid::parse_str(b.id.as_str()).unwrap()))
        }
    }
}

async fn create_storage_grant(
    client_id: Uuid,
    database: &Database,
    grant: &StorageGrant,
) -> Result<Uuid, GrantError> {
    match database.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            let grant_id: DbResult<BareId> = sqlx::query_as("INSERT INTO storage_grants (client_id, grant_id, allowed_storage) VALUES ($1::uuid, $2::uuid, $3) RETURNING CAST(id AS TEXT) as id;")
                .bind(client_id.to_string())
                .bind(grant.grant_id().to_string())
                .bind(grant.authorized_data_size() as i64)
                .fetch_one(conn)
                .await
                .map_err(postgres::map_sqlx_error);

            match grant_id {
                Ok(gid) => Ok(Uuid::parse_str(gid.id.as_str()).unwrap()),
                Err(DbError::RecordExists) => Err(GrantError::AlreadyRecorded),
                Err(err) => Err(GrantError::Database(err)),
            }
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

            let grant_id: DbResult<BareId> = sqlx::query_as("INSERT INTO storage_grants (client_id, grant_id, allowed_storage) VALUES ($1, $2, $3) RETURNING id;")
                .bind(client_id.to_string())
                .bind(grant.grant_id().to_string())
                .bind(grant.authorized_data_size() as i64)
                .fetch_one(conn)
                .await
                .map_err(sqlite::map_sqlx_error);

            match grant_id {
                Ok(gid) => Ok(Uuid::parse_str(gid.id.as_str()).unwrap()),
                Err(DbError::RecordExists) => Err(GrantError::AlreadyRecorded),
                Err(err) => Err(GrantError::Database(err)),
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GrantError {
    #[error("provided storage grant has already been recorded")]
    AlreadyRecorded,

    #[error("database issue occurred")]
    Database(#[from] DbError),
}

impl IntoResponse for GrantError {
    fn into_response(self) -> Response {
        use GrantError::*;

        match &self {
            AlreadyRecorded => {
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            Database(err) => {
                tracing::error!(error = ?err, "a database error occurred");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
