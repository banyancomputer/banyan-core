use std::path::PathBuf;

use axum::extract::FromRef;
use jsonwebtoken::{DecodingKey, EncodingKey};
use object_store::local::LocalFileSystem;
use openssl::ec::{EcGroup, EcKey};
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
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

    bucket_metadata_upload_directory: PathBuf,
    
    storage_host_url: String,
}

impl AppState {
    pub async fn from_config(config: &Config) -> Result<Self, StateError> {
        // Do a test setup to make sure the upload directory exists and is writable as an early
        // sanity check
        let bucket_metadata_upload_directory = config.bucket_metadata_upload_directory().clone();
        LocalFileSystem::new_with_prefix(&bucket_metadata_upload_directory)
            .map_err(StateError::inaccessible_upload_directory)?;

        let database_pool = database::setup(config.database_url()).await?;
        let (signing_key, verification_key) =
            load_or_create_service_key(config.signing_key_path())?;
        
        let storage_host_url = config.storage_host_url().to_string();

        Ok(Self {
            database_pool,
            signing_key,
            verification_key,
            bucket_metadata_upload_directory,
            storage_host_url,
        })
    }

    pub fn bucket_metadata_upload_directory(&self) -> &PathBuf {
        &self.bucket_metadata_upload_directory
    }

    pub fn storage_host_url(&self) -> &str {
        &self.storage_host_url
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
    let service_private_key = if !path.exists() {
        let ec_group =
            EcGroup::from_curve_name(Nid::SECP384R1).map_err(StateError::service_keygen_failed)?;

        let ec_key = EcKey::generate(&ec_group).map_err(StateError::service_keygen_failed)?;

        let pkey_private: PKey<Private> = ec_key
            .try_into()
            .map_err(StateError::service_keygen_failed)?;

        let private_pem = pkey_private
            .private_key_to_pem_pkcs8()
            .map_err(StateError::service_keygen_failed)?;

        std::fs::write(path, private_pem).map_err(StateError::write_service_key)?;

        pkey_private
    } else {
        let pem_bytes = std::fs::read(path).map_err(StateError::read_service_key)?;

        PKey::private_key_from_pem(pem_bytes.as_ref()).map_err(StateError::key_loading)?
    };

    let private_pem_bytes = service_private_key
        .private_key_to_pem_pkcs8()
        .map_err(StateError::key_loading)?;
    let encoding_key = EncodingKey::from_ec_pem(private_pem_bytes.as_ref())
        .map_err(StateError::loading_state_keys)?;

    let public_pem_bytes = service_private_key
        .public_key_to_pem()
        .map_err(StateError::key_loading)?;
    let decoding_key = DecodingKey::from_ec_pem(public_pem_bytes.as_ref())
        .map_err(StateError::loading_state_keys)?;

    Ok((encoding_key, decoding_key))
}
