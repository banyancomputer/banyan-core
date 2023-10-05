use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::FromRef;
use jwt_simple::prelude::*;
use object_store::local::LocalFileSystem;
use sqlx::sqlite::SqlitePool;

use crate::app::{Config, ProviderCredential, Secrets, ServiceSigningKey, ServiceVerificationKey};
use crate::database::{Database, DatabaseSetupError};

#[derive(Clone)]
pub struct State {
    database: Database,
    secrets: Secrets,
    service_verifier: ServiceVerificationKey,
    upload_directory: PathBuf,
}

impl State {
    pub fn database(&self) -> Database {
        self.database.clone()
    }

    pub async fn from_config(config: &Config) -> Result<Self, StateSetupError> {
        // Do a test setup to make sure the upload directory exists and is writable as an early
        // sanity check
        LocalFileSystem::new_with_prefix(&config.upload_directory())
            .map_err(StateSetupError::InaccessibleUploadDirectory)?;

        let database = database::setup(config.database_url()).await?;

        // wrap our key and verifier
        let service_key = load_or_create_service_key();
        let service_verifier = service_key.verifier();

        let mut credentials = BTreeMap::new();
        credentials.insert(
            Arc::from("google"),
            ProviderCredential::new(config.google_client_id(), config.google_client_secret()),
        );
        let secrets = Secrets::new(credentials, service_key);

        Ok(Self {
            database,
            secrets,
            service_verifier,
            upload_directory: config.upload_directory(),
        })
    }

    pub fn secrets(&self) -> &Secrets {
        &self.secrets
    }

    pub fn service_verifier(&self) -> &ServiceVerificationKey {
        &self.service_verifier
    }

    pub fn upload_directory(&self) -> PathBuf {
        self.upload_directory.clone()
    }
}

impl FromRef<State> for Database {
    fn from_ref(state: &State) -> Self {
        state.database()
    }
}

impl FromRef<State> for Secrets {
    fn from_ref(state: &State) -> Self {
        state.secrets()
    }
}

impl FromRef<State> for ServiceVerificationKey {
    fn from_ref(state: &State) -> Self {
        state.service_verifier()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StateSetupError {
    #[error("unable to access configured upload directory: {0}")]
    InaccessibleUploadDirectory(object_store::Error),

    #[error("failed to setup the database: {0}")]
    DatabaseSetupError(DatabaseSetupError),
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

fn load_or_create_service_key(private_path: &PathBuf) -> Result<ServiceCreationKey, StateSetupError> {
    let mut session_key_raw = if private_path.exists() {
        let key_bytes = std::fs::read(private_path).map_err(StateSetupError::UnreadableSessionKey)?;
        let private_pem = String::from_utf8_lossy(&key_bytes);

        ES384KeyPair::from_pem(&private_pem).map_err(StateSetupError::InvalidSessionKey)?
    } else {
        let new_key = ES384KeyPair::generate();
        let private_pem = new_key.to_pem().expect("fresh keys to export");

        std::fs::write(private_path, private_pem).map_err(StateSetupError::SessionKeyWriteFailed)?;

        let public_spki = new_key.public_key_to_pem().expect("fresh key to have public component");
        let mut public_path = private_path.clone();
        public_path.set_extension("public");
        std::fs::write(public_path, public_spki).map_err(StateSetupError::PublicKeyWriteFailed)?;

        new_key
    };

    let fingerprint = fingerprint_key(&session_key_raw);
    session_key_raw = session_key_raw.with_key_id(&fingerprint);

    let mut fingerprint_path = private_path.clone();
    fingerprint_path.set_extension("fingerprint");
    if !fingerprint_path.exists() {
        std::fs::write(fingerprint_path, fingerprint).map_err(StateSetupError::FingerprintWriteFailed)?;
    }

    Ok(SessionCreationKey(session_key_raw))
}
