use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::database::models::Client;
use crate::database::Database;
use crate::extractors::authenticated_client::{
    current_consumed_storage, AuthenticatedClient, AuthenticatedClientError,
};
use crate::extractors::storage_grant::{StorageGrant, StorageGrantError};

pub struct StorageOrClient {
    authenticated_client: AuthenticatedClient,
}

impl StorageOrClient {
    pub fn authorized_storage(&self) -> u64 {
        self.authenticated_client.authorized_storage()
    }

    pub fn consumed_storage(&self) -> u64 {
        self.authenticated_client.consumed_storage()
    }

    pub fn fingerprint(&self) -> String {
        self.authenticated_client.fingerprint()
    }

    pub fn id(&self) -> Uuid {
        self.authenticated_client.id()
    }

    pub fn platform_id(&self) -> Uuid {
        self.authenticated_client.platform_id()
    }

    pub fn storage_grant_id(&self) -> Uuid {
        self.authenticated_client.storage_grant_id()
    }

    pub fn remaining_storage(&self) -> u64 {
        self.authenticated_client.remaining_storage()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for StorageOrClient
where
    AuthenticatedClient: FromRequestParts<S, Rejection = AuthenticatedClientError>,
    StorageGrant: FromRequestParts<S, Rejection = StorageGrantError>,
    S: Send + Sync,
    Pool<Sqlite>: FromRef<S>,
{
    type Rejection = StorageGrantError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Ok(authenticated_client) =
            AuthenticatedClient::from_request_parts(parts, state).await
        {
            return Ok(StorageOrClient {
                authenticated_client,
            });
        }

        let storage = StorageGrant::from_request_parts(parts, state).await?;
        let database = Database::from_ref(state);
        let client =
            match Client::find_by_fingerprint(&database, storage.client_fingerprint()).await? {
                Some(client) => client,
                None => Err(StorageGrantError::ClientNotFound)?,
            };

        let id = Uuid::parse_str(&client.id).map_err(|_| StorageGrantError::InvalidUuid)?;

        let authenticated_client = AuthenticatedClient::builder()
            .id(id)
            .platform_id(storage.platform_id())
            .fingerprint(storage.client_fingerprint().to_string())
            .authorized_storage(storage.authorized_data_size() as u64)
            .consumed_storage(current_consumed_storage(&database, &client.id).await?)
            .storage_grant_id(storage.grant_id())
            .build()
            .map_err(|_| StorageGrantError::CouldNotBuildAuthenticatedClient)?;

        Ok(StorageOrClient {
            authenticated_client,
        })
    }
}
