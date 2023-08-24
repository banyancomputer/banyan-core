use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::GrantVerificationKey;

#[derive(Deserialize, Serialize)]
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
    GrantVerificationKey: FromRef<S>,
    S: Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let _verification_key = GrantVerificationKey::from_ref(state);

        let grant = StorageGrant {
            client_id: Uuid::new_v4(),
            client_fingerprint: "03:4a:11:4f:ff:08:83:ad:4a:fc:72:b3:ce:50:1b:df:d8:40:63:d7".to_string(),

            authorized_data_size: 100_000,
        };

        Ok(grant)
    }
}
