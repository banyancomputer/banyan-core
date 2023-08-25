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
use crate::extractors::key_validator;

// todo: will need a way for a client to refresh their storage grant
const MAXIMUM_GRANT_AGE: u64 = 900;

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

        let claims = verification_key
            .verify_token::<TokenAuthorizations>(&raw_token, Some(verification_options))
            .map_err(|err| Self::Rejection::ValidationFailed(err))?;

        // annoyingly jwt-simple doesn't use the correct encoding for this...
        match (claims.nonce, claims.custom.nonce) {
            (_, Some(nnc)) => {
                if nnc.len() < 12 {
                    return Err(Self::Rejection::InsufficientNonce);
                }
            }
            (Some(nnc), _) => {
                if nnc.len() < 12 {
                    return Err(Self::Rejection::InsufficientNonce);
                }
            }
            _ => return Err(Self::Rejection::NonceMissing),
        }

        let grant_subject = match claims.subject {
            Some(gs) => gs.clone(),
            None => return Err(Self::Rejection::SubjectMissing),
        };

        let (client_id, client_fingerprint) = match key_validator().captures(&grant_subject) {
            Some(matches) => {
                let id_str: &str = matches.get(1).expect("captures should be guaranteed").as_str();
                let id = Uuid::parse_str(id_str).expect("already validated the format");

                let finger_str: &str = matches.get(1).expect("captures should be guaranteed").as_str();

                (id, finger_str.to_owned())
            }
            None => return Err(Self::Rejection::SubjectInvalid),
        };

        // todo: need to take in the domain the provider will be running as to lookup expectedUsage
        // what we were authorized as but we can fake it for now by assuming we're the only one
        // present.
        let authorized_data_size = match claims.custom.capabilities.get("https://staging.storage.banyan.computer/") {
            Some(ads) => ads.authorized_amount,
            None => return Err(StorageGrantError::WrongTarget),
        };

        let grant = StorageGrant {
            client_id,
            client_fingerprint,

            authorized_data_size,
        };

        Ok(grant)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StorageGrantError {
    #[error("token's nonce was not sufficiently long")]
    InsufficientNonce,

    #[error("no bearer authorization header found in request")]
    MissingHeader(TypedHeaderRejection),

    #[error("token didn't include a nonce")]
    NonceMissing,

    #[error("subject contained in the storage token didn't match our format")]
    SubjectInvalid,

    #[error("storage token was missing a subject")]
    SubjectMissing,

    #[error("storage token grant failed validation")]
    ValidationFailed(jwt_simple::Error),

    #[error("storage token grant was not intended for this server")]
    WrongTarget,
}

impl IntoResponse for StorageGrantError {
    fn into_response(self) -> Response {
        use StorageGrantError::*;

        match &self {
            InsufficientNonce | NonceMissing | SubjectInvalid | SubjectMissing => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "storage grant was not accepted" });
                (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
            }
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
            WrongTarget => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "storage grant ticket is for another storage provider" });
                (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
struct TokenAuthorizations {
    #[serde(rename="cap")]
    capabilities: HashMap<String, Usage>,

    #[serde(rename="nnc")]
    nonce: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct Usage {
    #[serde(rename="expectedUsage")]
    authorized_amount: usize,
}
