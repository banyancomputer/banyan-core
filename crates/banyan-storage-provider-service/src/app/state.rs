use std::path::PathBuf;

use jwt_simple::prelude::*;
use object_store::local::LocalFileSystem;

use crate::app::{Config, Secrets};
use crate::database::{self, Database, DatabaseSetupError};
use crate::utils::{sha1_fingerprint_publickey, SigningKey, VerificationKey};

#[derive(Clone)]
pub struct State {
    /// Access to the database
    database: Database,
    /// Secrets
    secrets: Secrets,
    /// Key used to verify service tokens. See [`Secrets::service_signing_key`] for complimentary key.
    service_verifier: VerificationKey,
    /// Directory where uploaded files are stored
    upload_directory: PathBuf,
}

impl State {
    pub async fn from_config(config: &Config) -> Result<Self, StateSetupError> {
        // Do a test setup to make sure the upload directory exists and is writable as an early
        // sanity check
        LocalFileSystem::new_with_prefix(config.upload_directory())
            .map_err(StateSetupError::InaccessibleUploadDirectory)?;

        let database = database::connect(&config.database_url())
            .await
            .map_err(StateSetupError::DatabaseSetupError)?;

        let service_key = load_service_key(&config.service_key_path())?;
        let service_verifier = service_key.verifier();

        let secrets = Secrets::new(service_key);

        Ok(Self {
            database,
            secrets,
            service_verifier,
            upload_directory: config.upload_directory(),
        })
    }

    pub fn database(&self) -> Database {
        self.database.clone()
    }

    pub fn secrets(&self) -> Secrets {
        self.secrets.clone()
    }

    pub fn service_verifier(&self) -> VerificationKey { 
        self.service_verifier.clone()
    }

    pub fn upload_directory(&self) -> PathBuf {
        self.upload_directory.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StateSetupError {
    #[error("unable to access configured upload directory: {0}")]
    InaccessibleUploadDirectory(object_store::Error),

    #[error("private service key could not be loaded: {0}")]
    InvalidServiceKey(jwt_simple::Error),

    #[error("failed to setup the database: {0}")]
    DatabaseSetupError(#[from] DatabaseSetupError),

    #[error("failed to write fingerprint: {0}")]
    FingerprintWriteFailed(std::io::Error),

    #[error("failed to write public key: {0}")]
    PublicKeyWriteFailed(std::io::Error),

    #[error("unable to write generated service key: {0}")]
    ServiceKeyWriteFailed(std::io::Error),

    #[error("failed to read private service key: {0}")]
    UnreadableServiceKey(std::io::Error),
}

fn load_service_key(private_path: &PathBuf) -> Result<SigningKey, StateSetupError> {
    let key_bytes = std::fs::read(private_path).map_err(StateSetupError::UnreadableServiceKey)?;
    let private_pem = String::from_utf8_lossy(&key_bytes);

    let service_key_inner =
        ES384KeyPair::from_pem(&private_pem).map_err(StateSetupError::InvalidServiceKey)?;

    let fingerprint = sha1_fingerprint_publickey(&service_key_inner.public_key());
    let service_key_inner = service_key_inner.with_key_id(&fingerprint);

    Ok(SigningKey::new(service_key_inner))
}
