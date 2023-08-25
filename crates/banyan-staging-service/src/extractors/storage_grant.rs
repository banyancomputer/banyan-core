use std::collections::HashMap;

use axum::{async_trait, Json, RequestPartsExt};
use axum::extract::{FromRef, FromRequestParts, TypedHeader};
use axum::extract::rejection::TypedHeaderRejection;
use axum::headers::Authorization;
use axum::headers::authorization::Bearer;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::prelude::*;
use uuid::Uuid;

use crate::app::PlatformVerificationKey;

// todo: will need a way for a client to refresh their storage grant
const MAXIMUM_GRANT_AGE: u64 = 900;

#[derive(Deserialize, Serialize)]
struct TokenAuthorizations {
    #[serde(rename="cap")]
    capabilities: HashMap<String, usize>,
}

pub struct StorageGrant {
    client_id: Uuid,
    client_fingerprint: String,

    authorized_data_size: usize,
}

impl StorageGrant {
    fn authorized_data_size(&self) -> usize {
        self.authorized_data_size
    }

    fn client_id(&self) -> &Uuid {
        &self.client_id
    }

    fn client_fingerprint(&self) -> &str {
        &self.client_fingerprint
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for StorageGrant
where
    PlatformVerificationKey: FromRef<S>,
    S: Sync,
{
    type Rejection = StorageGrantError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|err| Self::Rejection::MissingHeader(err))?;

        let raw_token = bearer.token();

        let verification_options = VerificationOptions {
            accept_future: false,
            allowed_audiences: Some(HashSet::from_strings(&["banyan-staging"])),
            max_validity: Some(Duration::from_secs(MAXIMUM_GRANT_AGE)),
            time_tolerance: Some(Duration::from_secs(15)),
            ..Default::default()
        };

        let verification_key = PlatformVerificationKey::from_ref(state);

        let _claims = verification_key
            .verify_token::<TokenAuthorizations>(&raw_token, Some(verification_options))
            .map_err(|err| Self::Rejection::ValidationFailed(err))?;

        // todo: need to take in the domain the provider will be running as to lookup expectedUsage
        // what we were authorized as but we can fake it for now by assuming we're the only one
        // present.

        let grant = StorageGrant {
            client_id: Uuid::new_v4(),
            client_fingerprint: "03:4a:11:4f:ff:08:83:ad:4a:fc:72:b3:ce:50:1b:df:d8:40:63:d7".to_string(),

            authorized_data_size: 100_000,
        };

        Ok(grant)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StorageGrantError {
    #[error("no bearer authorization header found in request")]
    MissingHeader(TypedHeaderRejection),

    #[error("storage token grant failed validation")]
    ValidationFailed(jwt_simple::Error),
}

impl IntoResponse for StorageGrantError {
    fn into_response(self) -> Response {
        use StorageGrantError::*;

        match &self {
            MissingHeader(err) => {
                // todo: add sources as data event tag
                tracing::error!("{self}: {err}");
                let err_msg = serde_json::json!({ "msg": "storage grant was not accepted" });
                (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
            }
            ValidationFailed(err) => {
                // todo: add sources as data event tag
                tracing::error!("{self}: {err}");
                let err_msg = serde_json::json!({ "msg": "storage grant was not accepted" });
                (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
            }
        }
    }
}
