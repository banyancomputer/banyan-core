use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::either::Either;

use crate::database::models::BlockDetails;
use crate::extractors::authenticated_client::{AuthenticatedClient, AuthenticatedClientError};
use crate::extractors::platform_identity::{PlatformIdentity, PlatformIdentityError};

/// Enum encompassing Authentication Strategies for API requests
pub enum BlockReader {
    AuthenticatedClient(AuthenticatedClient),
    PlatformIdentity(PlatformIdentity),
}

impl BlockReader {
    pub fn can_read_block(&self, block: &BlockDetails) -> bool {
        match &self {
            BlockReader::AuthenticatedClient(client) => {
                block.platform_id == client.platform_id().to_string()
            }
            BlockReader::PlatformIdentity(_) => true,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for BlockReader
where
    AuthenticatedClient: FromRequestParts<S, Rejection = AuthenticatedClientError>,
    PlatformIdentity: FromRequestParts<S, Rejection = PlatformIdentityError>,
    S: Send + Sync,
{
    type Rejection = PlatformIdentityError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Attempt to extract an authenticated client, if that fails, try to extract a platform identity
        let either: Either<AuthenticatedClient, PlatformIdentity> =
            Either::from_request_parts(parts, state).await?;
        Ok(match either {
            Either::E1(client) => BlockReader::AuthenticatedClient(client),
            Either::E2(identity) => BlockReader::PlatformIdentity(identity),
        })
    }
}
