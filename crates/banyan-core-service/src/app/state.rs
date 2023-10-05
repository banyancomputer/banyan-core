use std::path::PathBuf;

use axum::extract::FromRef;
use jsonwebtoken::{DecodingKey, EncodingKey};
use object_store::local::LocalFileSystem;
use openssl::ec::{EcGroup, EcKey};
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use sqlx::sqlite::SqlitePool;

use crate::app::{Config, Secrets, SessionCreationKey, SessionVerificationKey};

#[derive(Clone)]
pub struct State {
    database_pool: SqlitePool,
    upload_directory: PathBuf,

    secrets: Secrets,
    session_verifier: SessionVerificationKey,
}

impl State {
    pub async fn from_config(config: &Config) -> Result<Self, StateError> {
        // Do a test setup to make sure the upload directory exists and is writable as an early
        // sanity check
        LocalFileSystem::new_with_prefix(&config.upload_directory())
            .map_err(StateError::inaccessible_upload_directory)?;

        let database_pool = database::setup(config.database_url()).await?;
        let (signing_key, verification_key) =
            load_or_create_service_key(config.signing_key_path())?;

        Ok(Self {
            database_pool,
            upload_directory: config.upload_directory(),
        })
    }

    pub fn metadata_upload_directory(&self) -> &PathBuf {
        &self.metadata_upload_directory
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

    // Write the public key next to the signing key
    let mut public_key_path = path.clone();
    public_key_path.set_extension("public");
    if !public_key_path.exists() {
        let public_pem_bytes = service_private_key
            .public_key_to_pem()
            .map_err(StateError::key_loading)?;

        std::fs::write(&public_key_path, public_pem_bytes)
            .map_err(StateError::write_service_key)?;
    }

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
