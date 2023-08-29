use std::collections::HashSet;
use std::sync::OnceLock;

use axum::{async_trait, Json, RequestPartsExt};
use axum::extract::{FromRef, FromRequestParts, TypedHeader};
use axum::extract::rejection::TypedHeaderRejection;
use axum::headers::Authorization;
use axum::headers::authorization::Bearer;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use http::request::Parts;
use jwt_simple::prelude::*;
use regex::Regex;
use sqlx::FromRow;
use uuid::Uuid;

use crate::database::{Database, Executor};
use crate::extractors::fingerprint_validator;

/// Defines the maximum length of time we consider any individual token valid in seconds. If the
/// expiration is still in the future, but it was issued more than this many seconds in the past
/// we'll reject the token even if its otherwise valid.
const MAXIMUM_TOKEN_AGE: u64 = 900;

pub struct AuthenticatedClient {
    platform_id: Uuid,
    fingerprint: String,

    authorized_storage: usize,
    consumed_storage: usize,
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

        let unvalidated_header = Token::decode_metadata(&raw_token).map_err(|err| Self::Rejection::CorruptHeader(err))?;
        let _key_id = match unvalidated_header.key_id() {
            Some(kid) if fingerprint_validator().is_match(kid) => kid.to_string(),
            Some(_) => return Err(Self::Rejection::InvalidKeyId),
            None => return Err(Self::Rejection::MissingKeyId),
        };

        let database = Database::from_ref(state);
        let _known_key = match database.ex() {
            Executor::Postgres(ref mut _conn) => {
                //"SELECT id, platform_id, public_key FROM clients WHERE fingerprint = $1;"
                //"SELECT allowed_storage FROM storage_grants WHERE client_id = $1 ORDER BY created_at DESC LIMIT 1;" // $1 is the id column from the last one
                //"SELECT SUM(COALESCE(final_size, reported_size)) AS consumed_storage FROM uploads WHERE client_id = $1;"
                todo!()
            }
            Executor::Sqlite(ref mut _conn) => {
                //"SELECT id, platform_id, public_key FROM clients WHERE fingerprint = $1;"
                //"SELECT allowed_storage FROM storage_grants WHERE client_id = $1 ORDER BY created_at DESC LIMIT 1;" // $1 is the id column from the last one
                //"SELECT SUM(COALESCE(final_size, reported_size)) AS consumed_storage FROM uploads WHERE client_id = $1;"
                todo!()
            }
        };

        let _verification_options = VerificationOptions {
            accept_future: false,
            allowed_audiences: Some(HashSet::from_strings(&["banyan-staging"])),
            max_validity: Some(Duration::from_secs(MAXIMUM_TOKEN_AGE)),
            time_tolerance: Some(Duration::from_secs(15)),
            ..Default::default()
        };

        Ok(AuthenticatedClient {
            platform_id: Uuid::new_v4(),
            fingerprint: "a fingerprint".to_string(),

            authorized_storage: 0,
            consumed_storage: 0,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthenticatedClientError {
    #[error("unable to decode bearer token metadata")]
    CorruptHeader(jwt_simple::Error),

    #[error("bearer token key ID does not conform to our expectations")]
    InvalidKeyId,

    #[error("authentication header wasn't present")]
    MissingHeader,

    #[error("no token key ID was provided")]
    MissingKeyId,
}

impl IntoResponse for AuthenticatedClientError {
    fn into_response(self) -> Response {
        use AuthenticatedClientError::*;

        match &self {
            CorruptHeader(_) | InvalidKeyId | MissingKeyId => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "invalid request" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            MissingHeader => {
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
