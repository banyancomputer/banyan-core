use std::collections::HashMap;

use axum::extract::rejection::TypedHeaderRejection;
use axum::extract::{FromRef, FromRequestParts, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Json, RequestPartsExt};
use jwt_simple::prelude::*;
use uuid::Uuid;

use crate::app::{PlatformVerificationKey, ServiceHostname, ServiceName};

use super::{paired_id_validator, MAXIMUM_TOKEN_AGE};

#[derive(Debug)]
pub struct StorageGrant {
    platform_id: Uuid,
    grant_id: Uuid,

    client_fingerprint: String,

    authorized_data_size: usize,
}

impl StorageGrant {
    pub fn authorized_data_size(&self) -> usize {
        self.authorized_data_size
    }

    pub fn grant_id(&self) -> Uuid {
        self.grant_id
    }

    pub fn platform_id(&self) -> Uuid {
        self.platform_id
    }

    pub fn client_fingerprint(&self) -> &str {
        &self.client_fingerprint
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for StorageGrant
where
    ServiceName: FromRef<S>,
    ServiceHostname: FromRef<S>,
    PlatformVerificationKey: FromRef<S>,
    S: Sync,
{
    type Rejection = StorageGrantError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(Self::Rejection::MissingHeader)?;

        let raw_token = bearer.token();
        let verification_options =
            VerificationOptions {
                accept_future: false,
                allowed_audiences: Some(
                    HashSet::from_strings(&[ServiceName::from_ref(state).to_string()])
                ),
                max_validity: Some(Duration::from_secs(MAXIMUM_TOKEN_AGE)),
                time_tolerance: Some(Duration::from_secs(15)),
                ..Default::default()
            };

        let verification_key = PlatformVerificationKey::from_ref(state);

        let claims = verification_key
            .verify_token::<TokenAuthorizations>(raw_token, Some(verification_options))
            .map_err(Self::Rejection::ValidationFailed)?;

        // annoyingly jwt-simple doesn't use the correct encoding for this... we can support both
        // though and maybe we can fix upstream so it follows the spec
        match (claims.nonce.as_ref(), claims.custom.nonce.as_ref()) {
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
            Some(gs) => gs,
            None => return Err(Self::Rejection::SubjectMissing),
        };

        let (platform_id, client_fingerprint) = match paired_id_validator().captures(&grant_subject)
        {
            Some(matches) => {
                let id_str: &str = matches
                    .get(1)
                    .expect("captures should be guaranteed")
                    .as_str();
                let id = Uuid::parse_str(id_str).expect("already validated the format");

                let finger_str: &str = matches
                    .get(2)
                    .expect("captures should be guaranteed")
                    .as_str();

                (id, finger_str.to_owned())
            }
            None => return Err(Self::Rejection::SubjectInvalid),
        };

        let hostname = ServiceHostname::from_ref(state).clone();

        // todo: need to take in the domain the provider will be running as to lookup expectedUsage
        // what we were authorized as but we can fake it for now by assuming we're the only one
        // present.
        let usage = match claims.custom.capabilities.get(&hostname.to_string()) {
            Some(u) => u,
            None => {
                tracing::info!("expecting hostname with value of {}", &hostname);
                tracing::error!("received valid storage grant but didn't authorize extra storage for this host: {:?}", claims.custom);
                return Err(Self::Rejection::WrongTarget);
            }
        };

        let grant_id =
            Uuid::parse_str(&usage.grant_id).map_err(|_| StorageGrantError::InvalidGrant)?;

        let grant = StorageGrant {
            platform_id,
            grant_id,

            client_fingerprint,

            authorized_data_size: usage.available_storage,
        };

        Ok(grant)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StorageGrantError {
    #[error("token's nonce was not sufficiently long")]
    InsufficientNonce,

    #[error("grant ID was not a valid format")]
    InvalidGrant,

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
            InsufficientNonce | InvalidGrant | NonceMissing | SubjectInvalid | SubjectMissing => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "storage grant was not accepted" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            MissingHeader(err) => {
                // todo: add sources as data event tag
                tracing::error!("{self}: {err}");
                let err_msg = serde_json::json!({ "msg": "storage grant was not accepted" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            ValidationFailed(err) => {
                // todo: add sources as data event tag
                tracing::error!("{self}: {err}");
                let err_msg = serde_json::json!({ "msg": "storage grant was not accepted" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            WrongTarget => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "storage grant ticket is for another storage provider" });
                (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct TokenAuthorizations {
    #[serde(rename = "cap")]
    capabilities: HashMap<String, Usage>,

    #[serde(rename = "nnc")]
    nonce: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Usage {
    available_storage: usize,
    grant_id: String,
}
