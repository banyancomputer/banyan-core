use std::collections::{HashMap, HashSet};

use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::{JWT_ALLOWED_CLOCK_DRIFT, STORAGE_TICKET_DURATION};

const TICKET_ISSUER: &str = "banyan-platform";

/// This struct represents the additional required claims that needs to be included in a signed JWT
/// to authorize clients to store data at remote storage hosts. The structure of these additional
/// claims was designed to be compatible with the expectations of the UCAN specification
/// (https://github.com/ucan-wg/spec).
#[derive(Deserialize, Serialize)]
pub struct StorageTicket {
    #[serde(rename = "cap")]
    capabilities: HashMap<String, StorageCapabilities>,
}

impl StorageTicket {
    pub fn with_capabilities(capabilities: HashMap<String, StorageCapabilities>) -> Self {
        Self { capabilities }
    }
}

pub struct StorageTicketBuilder {
    capabilities: HashMap<String, StorageCapabilities>,
    audience: HashSet<String>,
    subject: String,
}

impl StorageTicketBuilder {
    pub fn add_audience(&mut self, audience: String) {
        self.audience.insert(audience);
    }

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

    pub fn build(self) -> JWTClaims<StorageTicket> {
        let ticket = StorageTicket::with_capabilities(self.capabilities);

        let mut claims = Claims::with_custom_claims(ticket, STORAGE_TICKET_DURATION.into())
            .with_audiences(self.audience)
            .with_issuer(TICKET_ISSUER)
            .with_subject(self.subject)
            .invalid_before(Clock::now_since_epoch() - JWT_ALLOWED_CLOCK_DRIFT.into());

        claims.create_nonce();
        claims.issued_at = Some(Clock::now_since_epoch());

        claims
    }

    pub fn new(subject: String) -> Self {
        Self {
            subject,
            audience: HashSet::default(),
            capabilities: HashMap::default(),
        }
    }
}

/// These are the storage host specific details about a particular client. The grant_id is used as
/// an extra association measure to validate the ticket with the core platform and allow the core
/// platform to track which tickets have been redeemed and in turn are considered active (most
/// recently generated one that has been redeemed is considered the current one).
#[derive(Deserialize, Serialize)]
pub struct StorageCapabilities {
    /// The number of bytes that a client is allowed to store at the specific storage host
    /// associated with this capability.
    #[serde(rename = "available_storage")]
    authorized_amount: i64,

    /// A UUID matching the database identifier for a user's storage grant.
    grant_id: String,
}