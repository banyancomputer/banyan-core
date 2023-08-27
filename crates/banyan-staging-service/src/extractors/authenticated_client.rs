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
use uuid::Uuid;

use crate::database::Database;

/// Defines the maximum length of time we consider any individual token valid in seconds. If the
/// expiration is still in the future, but it was issued more than this many seconds in the past
/// we'll reject the token even if its otherwise valid.
const MAXIMUM_TOKEN_AGE: u64 = 900;

pub struct AuthenticatedClient {
    device_id: Uuid,
    fingerprint: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedClient
where
    Database: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthenticatedClientError;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Err(AuthenticatedClientError::Placeholder)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthenticatedClientError {
    #[error("placeholder till the code is fleshed out")]
    Placeholder,
}

impl IntoResponse for AuthenticatedClientError {
    fn into_response(self) -> Response {
        use AuthenticatedClientError::*;

        match &self {
            Placeholder => {
                tracing::error!("a placeholder error occurred extracting the authenticated client");
                let err_msg = serde_json::json!({ "msg": "here there be dragons" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
