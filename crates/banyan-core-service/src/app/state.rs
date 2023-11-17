use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::FromRef;
use jwt_simple::prelude::*;
use object_store::local::LocalFileSystem;

use crate::app::{
    Config, MailgunSigningKey, ProviderCredential, Secrets, ServiceSigningKey,
    ServiceVerificationKey,
};
use crate::database::{self, Database, DatabaseSetupError};
use crate::event_bus::EventBus;
use crate::utils::keys::sha1_fingerprint_publickey;

#[derive(Clone)]
pub struct State {
    database: Database,
    event_bus: EventBus,
    secrets: Secrets,
    service_verifier: ServiceVerificationKey,
    upload_directory: PathBuf,
}

impl State {
    pub fn database(&self) -> Database {
        self.database.clone()
    }

    pub fn event_bus(&self) -> EventBus {
        self.event_bus.clone()
    }

    pub async fn from_config(config: &Config) -> Result<Self, StateSetupError> {
        // Do a test setup to make sure the upload directory exists and is writable as an early
        // sanity check
        LocalFileSystem::new_with_prefix(config.upload_directory())
            .map_err(StateSetupError::InaccessibleUploadDirectory)?;

        let database = database::connect(&config.database_url()).await?;
        let event_bus = EventBus::new();

        let mailgun_signing_key = config.mailgun_signing_key().map(MailgunSigningKey::new);

        // TODO: This probably shouldn't create a new key
        // the --generate-auth workflow in other services is preferable heres
        let service_key = load_or_create_service_key(&config.session_key_path())?;
        let service_verifier = service_key.verifier();

        let mut credentials = BTreeMap::new();
        credentials.insert(
            Arc::from("google"),
            ProviderCredential::new(config.google_client_id(), config.google_client_secret()),
        );
        let secrets = Secrets::new(credentials, mailgun_signing_key, service_key);

        Ok(Self {
            database,
            event_bus,
            secrets,
            service_verifier,
            upload_directory: config.upload_directory(),
        })
    }

    pub fn secrets(&self) -> Secrets {
        self.secrets.clone()
    }

    pub fn service_verifier(&self) -> ServiceVerificationKey {
        self.service_verifier.clone()
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

fn load_or_create_service_key(
    private_path: &PathBuf,
) -> Result<ServiceSigningKey, StateSetupError> {
    let mut session_key_raw = if private_path.exists() {
        let key_bytes =
            std::fs::read(private_path).map_err(StateSetupError::UnreadableServiceKey)?;
        let private_pem = String::from_utf8_lossy(&key_bytes);

        ES384KeyPair::from_pem(&private_pem).map_err(StateSetupError::InvalidServiceKey)?
    } else {
        let new_key = ES384KeyPair::generate();
        let private_pem = new_key.to_pem().expect("fresh keys to export");

        std::fs::write(private_path, private_pem)
            .map_err(StateSetupError::ServiceKeyWriteFailed)?;

        let public_spki = new_key
            .public_key()
            .to_pem()
            .expect("fresh key to have public component");
        let mut public_path = private_path.clone();
        public_path.set_extension("public");
        std::fs::write(public_path, public_spki).map_err(StateSetupError::PublicKeyWriteFailed)?;

        new_key
    };

    let fingerprint = sha1_fingerprint_publickey(&session_key_raw.public_key());
    session_key_raw = session_key_raw.with_key_id(&fingerprint);

    let mut fingerprint_path = private_path.clone();
    fingerprint_path.set_extension("fingerprint");
    if !fingerprint_path.exists() {
        std::fs::write(fingerprint_path, fingerprint)
            .map_err(StateSetupError::FingerprintWriteFailed)?;
    }

    Ok(ServiceSigningKey::new(session_key_raw))
}
