use std::path::PathBuf;

use axum::extract::FromRef;
use jsonwebtoken::{DecodingKey, EncodingKey};
use object_store::local::LocalFileSystem;
use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P384_SHA384_FIXED_SIGNING};
use sqlx::sqlite::SqlitePool;

mod database;
mod state_error;

use crate::config::Config;
pub use state_error::StateError;

#[derive(Clone)]
pub struct AppState {
    database_pool: SqlitePool,

    signing_key: EncodingKey,
    verification_key: DecodingKey,

    upload_directory: PathBuf,
}

impl AppState {
    pub async fn from_config(config: &Config) -> Result<Self, StateError> {
        // Do a test setup to make sure the upload directory exists and is writable as an early
        // sanity check
        let upload_directory = config.upload_directory().clone();
        LocalFileSystem::new_with_prefix(&upload_directory)
            .map_err(StateError::inaccessible_upload_directory)?;

        let database_pool = database::setup(config.database_url()).await?;
        let (signing_key, verification_key) =
            load_or_create_service_key(config.signing_key_path())?;

        Ok(Self {
            database_pool,
            signing_key,
            verification_key,
            upload_directory,
        })
    }

    pub fn upload_directory(&self) -> &PathBuf {
        &self.upload_directory
    }
}

impl FromRef<AppState> for DecodingKey {
    fn from_ref(state: &AppState) -> Self {
        state.verification_key.clone()
    }
}

impl FromRef<AppState> for EncodingKey {
    fn from_ref(state: &AppState) -> Self {
        state.signing_key.clone()
    }
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.database_pool.clone()
    }
}

fn load_or_create_service_key(path: &PathBuf) -> Result<(EncodingKey, DecodingKey), StateError> {
    if !path.exists() {
        let rng = SystemRandom::new();
        let key = EcdsaKeyPair::generate_pkcs8(&ECDSA_P384_SHA384_FIXED_SIGNING, &rng)
            .map_err(|_| StateError::service_keygen_failed())?;

        let pem = pem::Pem::new("ECDSA PRIVATE KEY".to_string(), key.as_ref());

        std::fs::write(path, pem::encode(&pem)).map_err(StateError::write_service_key)?;
    };

    let service_key_bytes = std::fs::read(path).map_err(StateError::read_service_key)?;

    let service_key_pem =
        pem::parse(service_key_bytes).map_err(|_| StateError::invalid_service_key())?;

    if service_key_pem.tag() != "ECDSA PRIVATE KEY" {
        return Err(StateError::invalid_service_key());
    }

    let private_key_der_bytes = service_key_pem.into_contents();

    let ekp = EcdsaKeyPair::from_pkcs8(
        &ECDSA_P384_SHA384_FIXED_SIGNING,
        private_key_der_bytes.as_ref(),
    )
    .map_err(|_| StateError::invalid_service_key())?;

    let encoding_key = EncodingKey::from_ec_der(private_key_der_bytes.as_ref());
    let decoding_key = DecodingKey::from_ec_der(ekp.public_key().as_ref());

    Ok((encoding_key, decoding_key))
}
