use std::sync::Arc;

use jwt_simple::algorithms::ECDSAP384PublicKeyLike;
use jwt_simple::prelude::*;
use object_store::local::LocalFileSystem;
use sha2::Digest;

use crate::app::{Config, Error, GrantVerificationKey, PlatformAuthKey};
use crate::database::{config_database, Db};

#[derive(Clone)]
pub struct State {
    database: Db,
    grant_verification_key: GrantVerificationKey,
    platform_auth_key: PlatformAuthKey,
}

impl State {
    // not implemented as a From trait so it can be async
    pub async fn from_config(config: &Config) -> Result<Self, Error> {
        // Make sure our upload directory is present and at least readable, could do a test write
        // which wouldn't be a bad idea...
        let upload_directory = config.upload_directory().clone();
        LocalFileSystem::new_with_prefix(&upload_directory)
            .map_err(Error::inaccessible_upload_directory)?;

        // Configure the database instance we're going use
        let database = config_database(&config).await?;

        // Parse the platform authentication key (this will be used to communicate with the
        // metadata service).
        let key_bytes = std::fs::read(config.platform_auth_key_path()).map_err(Error::unreadable_key)?;
        let pem = String::from_utf8_lossy(&key_bytes);
        let auth_raw = ES384KeyPair::from_pem(&pem).map_err(Error::invalid_key)?;
        let fingerprint = fingerprint_key(&auth_raw);
        let platform_auth_key = auth_raw.with_key_id(&fingerprint);

        // Parse the public grant verification key (this will be the one coming from the platform)
        let key_bytes = std::fs::read(config.grant_verification_key_path()).map_err(Error::unreadable_key)?;
        let pem = String::from_utf8_lossy(&key_bytes);
        let grant_verification_key = ES384PublicKey::from_pem(&pem).map_err(Error::invalid_key)?;

        Ok(Self {
            database,
            grant_verification_key: GrantVerificationKey::new(grant_verification_key),
            platform_auth_key: PlatformAuthKey::new(platform_auth_key),
        })
    }
}

impl axum::extract::FromRef<State> for Db {
    fn from_ref(state: &State) -> Self {
        state.database.clone()
    }
}

impl axum::extract::FromRef<State> for GrantVerificationKey {
    fn from_ref(state: &State) -> Self {
        state.grant_verification_key.clone()
    }
}

impl axum::extract::FromRef<State> for PlatformAuthKey {
    fn from_ref(state: &State) -> Self {
        state.platform_auth_key.clone()
    }
}

fn fingerprint_key(keys: &ES384KeyPair) -> String {
    let public_key = keys.key_pair().public_key();
    let compressed_point = public_key.as_ref().to_encoded_point(true);

    let mut hasher = sha2::Sha256::new();
    hasher.update(compressed_point);
    let hashed_bytes = hasher.finalize();

    hashed_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
