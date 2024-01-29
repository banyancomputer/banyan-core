use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;

use super::{AdminIdentity, StorageProviderIdentity};
use crate::database::Database;
use crate::extractors::session_identity::SessionIdentityError;
use crate::extractors::storage_provider_identity::StorageProviderIdentityError;
use crate::extractors::SessionIdentity;

pub enum StorageOrAdminIdentity {
    AdminIdentity(AdminIdentity),
    StorageProviderIdentity(StorageProviderIdentity),
}

#[async_trait]
impl<S> FromRequestParts<S> for StorageOrAdminIdentity
where
    Database: FromRef<S>,
    SessionIdentity: FromRequestParts<S, Rejection = SessionIdentityError>,
    S: Send + Sync,
{
    type Rejection = SessionIdentityError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match AdminIdentity::from_request_parts(parts, state).await {
            Ok(admin) => Ok(StorageOrAdminIdentity::AdminIdentity(admin)),
            Err(SessionIdentityError::NotAdmin(not_admin)) => {
                StorageProviderIdentity::from_request_parts(parts, state)
                    .await
                    .map(StorageOrAdminIdentity::StorageProviderIdentity)
                    .or_else(|e| match e {
                        StorageProviderIdentityError::MissingHeader(_) => {
                            Err(SessionIdentityError::NotAdmin(not_admin))
                        }
                        _ => Err(e.into()),
                    })
            }
            Err(e) => Err(e),
        }
    }
}

impl From<StorageProviderIdentityError> for SessionIdentityError {
    fn from(error: StorageProviderIdentityError) -> Self {
        use StorageProviderIdentityError::*;
        match error {
            BadKeyFormat | FormatError(_) => SessionIdentityError::EncodingError,
            DatabaseUnavailable(e) => SessionIdentityError::LookupFailed(e),
            StorageHostNotFound | MismatchedSubject | MissingHeader(_) | UnidentifiedKey => {
                SessionIdentityError::NoSession(error.to_string())
            }
            ExtremeTokenValidity | NeverValid => SessionIdentityError::SessionExpired,
        }
    }
}
