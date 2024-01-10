use std::collections::BTreeMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::FromRef;
use jwt_simple::prelude::*;
use object_store::local::LocalFileSystem;

use crate::app::{
    AdminServiceVerificationKey, Config, MailgunSigningKey, ProviderCredential, Secrets,
    ServiceKey, ServiceVerificationKey,
};
use crate::database::{self, Database, DatabaseSetupError};
use crate::event_bus::EventBus;
use crate::utils::keys::fingerprint_public_key;

#[derive(Clone)]
pub struct State {
    database: Database,
    event_bus: EventBus,
    secrets: Secrets,
    service_name: String,
    service_verifier: ServiceVerificationKey,
    upload_directory: PathBuf,

    /// The unique name of the admin service
    admin_service_name: String,
    /// Key used to verify admin service tokens.
    admin_service_verification_key: AdminServiceVerificationKey,
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

        let service_key = load_or_create_service_key(&config.service_key_path())?;
        let service_verifier = service_key.verifier();

        let mut credentials = BTreeMap::new();
        credentials.insert(
            Arc::from("google"),
            ProviderCredential::new(config.google_client_id(), config.google_client_secret()),
        );
        let secrets = Secrets::new(credentials, mailgun_signing_key, service_key);

        let admin_service_verification_key =
            load_admin_service_verification_key(&config.admin_service_public_key_path())?;

        Ok(Self {
            database,
            event_bus,
            secrets,
            service_name: config.service_name().to_string(),
            service_verifier,
            upload_directory: config.upload_directory(),
            admin_service_name: config.admin_service_name().to_string(),
            admin_service_verification_key,
        })
    }

    pub fn secrets(&self) -> Secrets {
        self.secrets.clone()
    }

    pub fn service_name(&self) -> &str {
        self.service_name.as_str()
    }

    pub fn service_verifier(&self) -> ServiceVerificationKey {
        self.service_verifier.clone()
    }

    pub fn upload_directory(&self) -> PathBuf {
        self.upload_directory.clone()
    }

    pub fn admin_service_name(&self) -> &str {
        &self.admin_service_name
    }

    pub fn admin_service_verification_key(&self) -> AdminServiceVerificationKey {
        self.admin_service_verification_key.clone()
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

    #[error("failed to read public admin service key: {0}")]
    AdminServiceKeyRead(std::io::Error),

    #[error("public admin service key could not be loaded: {0}")]
    InvalidAdminServiceKey(jwt_simple::Error),
}

fn load_or_create_service_key(private_path: &PathBuf) -> Result<ServiceKey, StateSetupError> {
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

    let fingerprint = fingerprint_public_key(&session_key_raw.public_key());
    session_key_raw = session_key_raw.with_key_id(&fingerprint);

    let mut fingerprint_path = private_path.clone();
    fingerprint_path.set_extension("fingerprint");
    if !fingerprint_path.exists() {
        std::fs::write(fingerprint_path, fingerprint)
            .map_err(StateSetupError::FingerprintWriteFailed)?;
    }

    Ok(ServiceKey::new(session_key_raw))
}
fn load_admin_service_verification_key(
    path: &PathBuf,
) -> Result<AdminServiceVerificationKey, StateSetupError> {
    let key_bytes = std::fs::read(path).map_err(StateSetupError::AdminServiceKeyRead)?;
    let public_pem = String::from_utf8_lossy(&key_bytes);

    let admin_service_verification_key_inner =
        ES384PublicKey::from_pem(&public_pem).map_err(StateSetupError::InvalidAdminServiceKey)?;

    // TODO: use normalized fingerprint -- blake3
    let fingerprint = fingerprint_public_key(&admin_service_verification_key_inner);
    let admin_service_verification_key_inner =
        admin_service_verification_key_inner.with_key_id(&fingerprint);

    Ok(AdminServiceVerificationKey::new(
        admin_service_verification_key_inner,
    ))
}

#[cfg(test)]
pub mod test {
    use std::path::PathBuf;
    use std::sync::Arc;

    use axum::extract::State;
    use jwt_simple::algorithms::ES384KeyPair;

    use crate::app::{
        AdminServiceVerificationKey, AppState, ProviderCredential, Secrets, ServiceKey,
        ServiceVerificationKey,
    };
    use crate::database::Database;
    use crate::event_bus::EventBus;

    pub fn mock_app_state(database: Database) -> State<AppState> {
        let mut provider_creds = std::collections::BTreeMap::new();
        provider_creds.insert(
            Arc::from("mock_provider"),
            ProviderCredential::new("mock_pem", "secret"),
        );
        State(AppState {
            database,
            event_bus: EventBus::default(),
            secrets: Secrets::new(
                provider_creds,
                None,
                ServiceKey::new(ES384KeyPair::generate()),
            ),
            service_name: "mock_service".to_string(),
            service_verifier: ServiceVerificationKey::new(ES384KeyPair::generate().public_key()),
            upload_directory: PathBuf::from("/mock/path"),

            admin_service_name: "mock_admin_service".to_string(),
            admin_service_verification_key: AdminServiceVerificationKey::new(
                ES384KeyPair::generate().public_key(),
            ),
        })
    }
}
