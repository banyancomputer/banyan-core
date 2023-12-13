use std::collections::{HashMap, HashSet};

use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::{JWT_ALLOWED_CLOCK_DRIFT, STORAGE_TICKET_DURATION};
use crate::database::models::SelectedStorageHost;

#[derive(Default, Deserialize, Serialize)]
pub struct StorageTicket {
    #[serde(rename = "cap")]
    capabilities: HashMap<String, StorageCapabilities>,

    #[serde(skip_serializing, default)]
    audience: HashSet<String>,
}

impl StorageTicket {
    pub fn add_authorization(
        &mut self,
        grant_id: String,
        storage_host_url: String,
        authorized_amount: i64,
    ) {
        let caps = StorageCapabilities {
            authorized_amount,
            grant_id,
        };
        self.capabilities.insert(storage_host_url, caps);
    }
}

#[derive(Deserialize, Serialize)]
struct StorageCapabilities {
    #[serde(rename = "available_storage")]
    authorized_amount: i64,
    grant_id: String,
}

pub fn generate_storage_claim(
    subject: String,
    grant_id: String,
    storage_host: &SelectedStorageHost,
    authorized_amount: i64,
) -> JWTClaims<StorageTicket> {
    let mut ticket = StorageTicket::default();
    ticket.add_authorization(grant_id, storage_host.url, authorized_amount);

    let mut claims = Claims::with_custom_claims(ticket, STORAGE_TICKET_DURATION.into())
        .with_audiences(HashSet::from_strings(&[storage_host.name.as_str()]))
        .with_issuer("banyan-platform")
        .with_subject(subject)
        .invalid_before(Clock::now_since_epoch() - JWT_ALLOWED_CLOCK_DRIFT.into());

    claims.create_nonce();
    claims.issued_at = Some(Clock::now_since_epoch());

    claims
}
