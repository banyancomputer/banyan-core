use serde::{Deserialize, Serialize};

use crate::database::Database;
use crate::extractors::authenticated_client::{AuthenticatedClientError, RemoteId};

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct Clients {
    pub id: String,
    pub platform_id: String,
    pub fingerprint: String,
    pub public_key: String,
}

impl Clients {
    pub async fn find_by_upload_id(conn: &Database, upload_id: &str) -> Result<Self, sqlx::Error> {
        let client = sqlx::query_as!(
            Clients,
            "SELECT c.id, c.platform_id, c.fingerprint, c.public_key FROM clients AS c
              JOIN uploads u on c.id = u.client_id
            WHERE u.id = $1;",
            upload_id
        )
        .fetch_one(conn)
        .await?;
        Ok(client)
    }

    pub async fn id_from_fingerprint(
        db: &Database,
        fingerprint: &str,
    ) -> Result<RemoteId, AuthenticatedClientError> {
        let maybe_remote_id: Option<RemoteId> = sqlx::query_as(
            "SELECT id, platform_id, public_key FROM clients WHERE fingerprint = $1;",
        )
        .bind(fingerprint)
        .fetch_optional(db)
        .await
        .map_err(AuthenticatedClientError::DbFailure)?;

        match maybe_remote_id {
            Some(id) => Ok(id),
            None => Err(AuthenticatedClientError::UnknownFingerprint),
        }
    }
}
