use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::database::{map_sqlx_error, BareId, Database, DatabaseError, DbResult};
use crate::extractors::StorageGrant;

#[derive(Deserialize, Serialize)]
pub struct GrantRequest {
    public_key: String,
}

pub async fn handler(
    State(state): State<AppState>,
    grant: StorageGrant,
    Json(request): Json<GrantRequest>,
) -> Result<Response, GrantError> {
    let database = state.database();
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
    let client_id: String = sqlx::query_scalar("INSERT INTO clients (platform_id, fingerprint, public_key) VALUES ($1, $2, $3) RETURNING id;")
                .bind(grant.platform_id().to_string())
                .bind(grant.client_fingerprint())
                .bind(request.public_key)
                .fetch_one(database)
                .await
                .map_err(map_sqlx_error)
                .map_err(GrantError::Database)?;

    Ok(Uuid::parse_str(&client_id).unwrap())
}

async fn existing_grant_user(
    database: &Database,
    grant: &StorageGrant,
) -> Result<Option<Uuid>, GrantError> {
    let user_id: Option<BareId> = sqlx::query_as("SELECT id FROM clients WHERE fingerprint = $1")
        .bind(grant.client_fingerprint())
        .fetch_optional(database)
        .await
        .map_err(map_sqlx_error)
        .map_err(GrantError::Database)?;

    Ok(user_id.map(|b| Uuid::parse_str(b.id.as_str()).unwrap()))
}

async fn create_storage_grant(
    client_id: Uuid,
    database: &Database,
    grant: &StorageGrant,
) -> Result<Uuid, GrantError> {
    let grant_id: DbResult<BareId> = sqlx::query_as("INSERT INTO storage_grants (client_id, grant_id, allowed_storage) VALUES ($1, $2, $3) RETURNING id;")
                .bind(client_id.to_string())
                .bind(grant.grant_id().to_string())
                .bind(grant.authorized_data_size() as i64)
                .fetch_one(database)
                .await
                .map_err(map_sqlx_error);

    match grant_id {
        Ok(gid) => Ok(Uuid::parse_str(gid.id.as_str()).unwrap()),
        Err(DatabaseError::RecordExists) => Err(GrantError::AlreadyRecorded),
        Err(err) => Err(GrantError::Database(err)),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GrantError {
    #[error("provided storage grant has already been recorded")]
    AlreadyRecorded,

    #[error("database issue occurred")]
    Database(#[from] DatabaseError),
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
