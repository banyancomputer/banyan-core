use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::extractors::SigningKey;

const STORAGE_TICKET_DURATION: u64 = 15 * 60; // 15 minutes
const STORAGE_TICKET_NONCE_LENGTH: usize = 32;
const STORAGE_TICKET_LEEWAY: u64 = 20; // 20 seconds

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Claims {
    #[serde(rename = "iat")]
    pub issued_at: u64,

    #[serde(rename = "nnc")]
    pub nonce: String,

    #[serde(rename = "exp")]
    pub expiration: u64,

    #[serde(rename = "nbf")]
    pub not_before: u64,

    #[serde(rename = "aud")]
    pub audience: String,

    #[serde(rename = "sub")]
    pub subject: String,

    #[serde(rename = "cap")]
    pub capabilities: Map<String, Value>,
}

/// Generate a JWT that acts as a storage ticket
/// # Arguments
/// * account_id - The account id to use for the subject. Represents the account that requested the storage ticket
/// * fingerprint - The fingerprint to use for the subject. Represents the key that authenticated the request for the storage ticket
/// * storage_host_name - The storage host name to use for the audience. Represents the storage host that the storage ticket is for
/// * storage_host_url - The storage host url to use for the claim.
/// * current_usage - The current usage of the account that requested the storage ticket
/// * expected_increase - The expected increase in usage for the storage ticket (in bytes)
/// * signing_key - The signing key to use for the storage ticket
/// # Returns
/// The storage ticket as a JWT string
pub fn generate_storage_ticket(
    account_id: &str,
    fingerprint: &str,
    storage_host_name: &str,
    storage_host_url: &str,
    available_storage: u64,
    signing_key: &SigningKey,
) -> Result<String, jsonwebtoken::errors::Error> {
    let mut available_storage_map = Map::new();
    available_storage_map.insert("availableStorage".to_string(), available_storage.into());
    let mut capabilities = Map::new();
    capabilities.insert(storage_host_url.to_string(), available_storage_map.into());
    let nonce = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(STORAGE_TICKET_NONCE_LENGTH)
        .map(char::from)
        .collect::<String>();
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES384);
    let claims = Claims {
        issued_at: chrono::Utc::now().timestamp() as u64,
        nonce,
        expiration: (chrono::Utc::now() + chrono::Duration::seconds(STORAGE_TICKET_DURATION as i64))
            .timestamp() as u64,
        not_before: (chrono::Utc::now() - chrono::Duration::seconds(STORAGE_TICKET_LEEWAY as i64))
            .timestamp() as u64,
        audience: storage_host_name.to_string(),
        subject: format!("{}@{}", account_id, fingerprint),
        capabilities,
    };
    jsonwebtoken::encode(&header, &claims, &signing_key.0)
}
