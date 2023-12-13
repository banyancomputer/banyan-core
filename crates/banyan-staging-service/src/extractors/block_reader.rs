use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::either::Either;

use crate::database::models::BlockDetails;
use crate::extractors::authenticated_client::{AuthenticatedClient, AuthenticatedClientError};
use crate::extractors::platform_identity::{PlatformIdentity, PlatformIdentityError};

/// Enum encompassing Authentication Strategies for API requests
pub struct BlockReader(Either<AuthenticatedClient, PlatformIdentity>);

impl BlockReader {
    pub fn can_read_block(&self, block: &BlockDetails) -> bool {
        match &self.0 {
            // If this is an authenticated client, we need to make sure they own the block they're trying to retrieve
            Either::E1(client) => block.platform_id == client.platform_id().to_string(),
            // Otherwise just let the platform identity pass through
            Either::E2(_) => true,
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
        Ok(BlockReader(either))
    }
}
