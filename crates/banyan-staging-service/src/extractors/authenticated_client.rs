use std::collections::HashSet;
use std::sync::OnceLock;

use axum::extract::rejection::TypedHeaderRejection;
use axum::extract::{FromRef, FromRequestParts, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Json, RequestPartsExt};
use http::request::Parts;
use jwt_simple::prelude::*;
use regex::Regex;
use sqlx::FromRow;
use uuid::Uuid;

use crate::database::{Database, DbError, Executor};
use crate::extractors::fingerprint_validator;

/// Defines the maximum length of time we consider any individual token valid in seconds. If the
/// expiration is still in the future, but it was issued more than this many seconds in the past
/// we'll reject the token even if its otherwise valid.
const MAXIMUM_TOKEN_AGE: u64 = 900;

pub struct AuthenticatedClient {
    id: Uuid,

    platform_id: Uuid,
    fingerprint: String,

    authorized_storage: u64,
    consumed_storage: u64,
}

impl AuthenticatedClient {
    pub fn authorized_storage(&self) -> u64 {
        self.authorized_storage
    }

    pub fn consumed_storage(&self) -> u64 {
        self.consumed_storage
    }

    pub fn fingerprint(&self) -> String {
        self.fingerprint.clone()
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn platform_id(&self) -> Uuid {
        self.platform_id
    }

    pub fn remaining_storage(&self) -> u64 {
        self.authorized_storage - self.consumed_storage
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedClient
where
    Database: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthenticatedClientError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| Self::Rejection::MissingHeader)?;

        let raw_token = bearer.token();

        let unvalidated_header =
            Token::decode_metadata(raw_token).map_err(Self::Rejection::CorruptHeader)?;
        let key_id = match unvalidated_header.key_id() {
            Some(kid) if fingerprint_validator().is_match(kid) => kid.to_string(),
            Some(_) => return Err(Self::Rejection::InvalidKeyId),
            None => return Err(Self::Rejection::MissingKeyId),
        };

        let database = Database::from_ref(state);

        let client_id = id_from_fingerprint(&database, &key_id).await?;
        let client_verification_key = ES384PublicKey::from_pem(&client_id.public_key)
            .map_err(Self::Rejection::CorruptDatabaseKey)?;

        let verification_options = VerificationOptions {
            accept_future: false,
            // TODO: this might not be a quite right, but it's probably fine for now
            //allowed_audiences: Some(HashSet::from_strings(&["banyan-platform"])),
            max_validity: Some(Duration::from_secs(MAXIMUM_TOKEN_AGE)),
            time_tolerance: Some(Duration::from_secs(15)),
            ..Default::default()
        };

        let claims = client_verification_key
            .verify_token::<TokenClaims>(raw_token, Some(verification_options))
            .map_err(Self::Rejection::ValidationFailed)?;

        // annoyingly jwt-simple doesn't use the correct encoding for this... we can support both
        // though and maybe we can fix upstream so it follows the spec
        let nonce = claims
            .custom
            .nonce
            .or(claims.nonce)
            .ok_or(Self::Rejection::BadNonce)?;
        if nonce.len() < 12 {
            return Err(Self::Rejection::BadNonce);
        }

        let authorized_storage = current_authorized_storage(&database, &client_id.id).await?;
        let consumed_storage = current_consumed_storage(&database, &client_id.id).await?;

        let internal_id = match Uuid::parse_str(&client_id.id) {
            Ok(pi) => pi,
            Err(err) => return Err(Self::Rejection::CorruptInternalId(err)),
        };

        let platform_id = match Uuid::parse_str(&client_id.platform_id) {
            Ok(pi) => pi,
            Err(err) => return Err(Self::Rejection::CorruptPlatformId(err)),
        };

        Ok(AuthenticatedClient {
            id: internal_id,

            platform_id,
            fingerprint: key_id,

            authorized_storage,
            consumed_storage,
        })
    }
}

pub async fn current_authorized_storage(
    db: &Database,
    client_id: &str,
) -> Result<u64, AuthenticatedClientError> {
    match db.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            let maybe_allowed_storage: Option<i32> = sqlx::query_scalar(
                "SELECT allowed_storage FROM storage_grants WHERE client_id = $1::uuid ORDER BY created_at DESC LIMIT 1;",
            )
            .bind(client_id)
            .fetch_optional(conn)
            .await
            .map_err(postgres::map_sqlx_error)
            .map_err(AuthenticatedClientError::DbFailure)?;

            Ok(maybe_allowed_storage.unwrap_or(0) as u64)
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

            let maybe_allowed_storage: Option<i64> = sqlx::query_scalar(
                "SELECT allowed_storage FROM storage_grants WHERE client_id = $1 ORDER BY created_at DESC LIMIT 1;",
            )
            .bind(client_id)
            .fetch_optional(conn)
            .await
            .map_err(sqlite::map_sqlx_error)
            .map_err(AuthenticatedClientError::DbFailure)?;

            Ok(maybe_allowed_storage.unwrap_or(0) as u64)
        }
    }
}

pub async fn current_consumed_storage(
    db: &Database,
    client_id: &str,
) -> Result<u64, AuthenticatedClientError> {
    match db.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            let maybe_consumed_storage: Option<i64> = sqlx::query_scalar(
                "SELECT SUM(COALESCE(final_size, reported_size)) AS consumed_storage FROM uploads WHERE client_id = $1::uuid;",
            )
            .bind(client_id)
            .fetch_optional(conn)
            .await
            .map_err(postgres::map_sqlx_error)
            .map_err(AuthenticatedClientError::DbFailure)?;

            Ok(maybe_consumed_storage.unwrap_or(0) as u64)
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

            let maybe_consumed_storage: Option<i64> = sqlx::query_scalar(
                "SELECT SUM(COALESCE(final_size, reported_size)) AS consumed_storage FROM uploads WHERE client_id = $1;",
            )
            .bind(client_id)
            .fetch_optional(conn)
            .await
            .map_err(sqlite::map_sqlx_error)
            .map_err(AuthenticatedClientError::DbFailure)?;

            Ok(maybe_consumed_storage.unwrap_or(0) as u64)
        }
    }
}

pub async fn id_from_fingerprint(
    db: &Database,
    fingerprint: &str,
) -> Result<RemoteId, AuthenticatedClientError> {
    match db.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            let maybe_remote_id: Option<RemoteId> = sqlx::query_as(
                "SELECT CAST(id AS TEXT) as id, CAST(platform_id AS TEXT) as platform_id, public_key FROM clients WHERE fingerprint = $1;",
            )
            .bind(fingerprint)
            .fetch_optional(conn)
            .await
            .map_err(postgres::map_sqlx_error)
            .map_err(AuthenticatedClientError::DbFailure)?;

            match maybe_remote_id {
                Some(id) => Ok(id),
                None => Err(AuthenticatedClientError::UnknownFingerprint),
            }
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

            let maybe_remote_id: Option<RemoteId> = sqlx::query_as(
                "SELECT id, platform_id, public_key FROM clients WHERE fingerprint = $1;",
            )
            .bind(fingerprint)
            .fetch_optional(conn)
            .await
            .map_err(sqlite::map_sqlx_error)
            .map_err(AuthenticatedClientError::DbFailure)?;

            match maybe_remote_id {
                Some(id) => Ok(id),
                None => Err(AuthenticatedClientError::UnknownFingerprint),
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthenticatedClientError {
    #[error("nonce wasn't present or insufficiently long")]
    BadNonce,

    #[error("unable to authenticate user as the key associated with the fingerprint isn't valid")]
    CorruptDatabaseKey(jwt_simple::Error),

    #[error("unable to decode bearer token metadata")]
    CorruptHeader(jwt_simple::Error),

    #[error("database internal ID wasn't a valid UUID")]
    CorruptInternalId(uuid::Error),

    #[error("database platform ID wasn't a valid UUID")]
    CorruptPlatformId(uuid::Error),

    #[error("an unexpected database failure before the authentication could be verified")]
    DbFailure(DbError),

    #[error("bearer token key ID does not conform to our expectations")]
    InvalidKeyId,

    #[error("authentication header wasn't present")]
    MissingHeader,

    #[error("no token key ID was provided")]
    MissingKeyId,

    #[error("provided key fingerprint is not present in the database")]
    UnknownFingerprint,

    #[error("failed to validate JWT with provided key and parameters")]
    ValidationFailed(jwt_simple::Error),
}

impl IntoResponse for AuthenticatedClientError {
    fn into_response(self) -> Response {
        use AuthenticatedClientError::*;

        match &self {
            BadNonce | CorruptHeader(_) | InvalidKeyId | MissingKeyId | ValidationFailed(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "invalid request" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            CorruptDatabaseKey(_) | CorruptInternalId(_) | CorruptPlatformId(_) | DbFailure(_) => {
                tracing::error!("{self}");
                let err_msg =
                    serde_json::json!({ "msg": "service is experiencing internal issues" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            MissingHeader | UnknownFingerprint => {
                let err_msg = serde_json::json!({ "msg": "authentication required" });
                (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
            }
        }
    }
}

#[derive(FromRow)]
pub struct ClientKey {
    public_key: String,
}

#[derive(FromRow, Debug)]
pub struct RemoteId {
    id: String,
    platform_id: String,
    public_key: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TokenClaims {
    #[serde(rename = "nnc")]
    nonce: Option<String>,
}
