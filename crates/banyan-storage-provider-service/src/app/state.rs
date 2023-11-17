use std::path::PathBuf;
use std::ops::Deref;

use axum::extract::FromRef;
use jwt_simple::prelude::*;
use object_store::local::LocalFileSystem;
use url::Url;

use crate::app::{Config, Secrets};
use crate::database::{self, Database, DatabaseSetupError};
use crate::utils::{sha1_fingerprint_publickey, fingerprint_key_pair, SigningKey, VerificationKey};

// Helper struct for extracting state from requests
pub struct ServiceName(String);
pub struct ServiceHostname(Url);
pub struct ServiceVerificationKey(VerificationKey);
pub struct PlatformName(String);
pub struct PlatformHostname(Url);
pub struct PlatformVerificationKey(VerificationKey);

#[derive(Clone)]
pub struct State {
    // Resources

    /// Access to the database
    database: Database,
    /// Directory where uploaded files are stored
    upload_directory: PathBuf,

    // Secrets

    /// All runtime secrets
    secrets: Secrets,

    // Service identity

    /// The unique name of the service, as registered with the platform
    service_name: String, 
    /// The hostname of the service
    service_hostname: Url,
    /// Key used to verify service tokens. See [`Secrets::service_signing_key`] for complimentary key.
    service_verification_key: VerificationKey,

    // Platform authentication

    /// The unique name of the platform
    platform_name: String,
    /// The hostname of the platform
    platform_hostname: Url,
    /// Key used to verify platform tokens.
    platform_verification_key: VerificationKey,
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

        let service_signing_key = load_service_key(&config.service_key_path())?;
        let service_verification_key = service_signing_key.verifier();

        let platform_verification_key = load_platform_verfication_key(&config.platform_verification_key_path())?;

        let secrets = Secrets::new(service_signing_key);

        Ok(Self {
            database,
            upload_directory: config.upload_directory(),

            secrets,
            
            service_name: config.service_name().to_string(),
            service_hostname: config.service_hostname().clone(),
            service_verification_key,

            platform_name: config.platform_name().to_string(),
            platform_hostname: config.platform_hostname().clone(),
            platform_verification_key,
        })
    }

    pub fn database(&self) -> Database {
        self.database.clone()
    }

    pub fn upload_directory(&self) -> PathBuf {
        self.upload_directory.clone()
    }

    pub fn secrets(&self) -> Secrets {
        self.secrets.clone()
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn service_hostname(&self) -> Url {
        self.service_hostname.clone()
    }

    pub fn service_verification_key(&self) -> VerificationKey {
        self.service_verification_key.clone()
    }

    pub fn platform_name(&self) -> &str {
        &self.platform_name
    }

    pub fn platform_hostname(&self) -> Url {
        self.platform_hostname.clone()
    }

    pub fn platform_verification_key(&self) -> VerificationKey {
        self.platform_verification_key.clone()
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

impl FromRef<State> for ServiceName {
    fn from_ref(state: &State) -> Self {
        ServiceName(state.service_name().to_string())
    }
}

impl FromRef<State> for ServiceHostname {
    fn from_ref(state: &State) -> Self {
        ServiceHostname(state.service_hostname().clone())
    }
}

impl FromRef<State> for ServiceVerificationKey {
    fn from_ref(state: &State) -> Self {
        ServiceVerificationKey(state.service_verification_key().clone())
    }
}

impl FromRef<State> for PlatformName {
    fn from_ref(state: &State) -> Self {
        PlatformName(state.platform_name().to_string())
    }
}

impl FromRef<State> for PlatformHostname {
    fn from_ref(state: &State) -> Self {
        PlatformHostname(state.platform_hostname().clone())
    }
}

impl FromRef<State> for PlatformVerificationKey {
    fn from_ref(state: &State) -> Self {
        PlatformVerificationKey(state.platform_verification_key().clone())
    }
}

impl Deref for ServiceName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for ServiceHostname {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for ServiceVerificationKey {
    type Target = VerificationKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for PlatformName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for PlatformHostname {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for PlatformVerificationKey {
    type Target = VerificationKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StateSetupError {
    #[error("unable to access configured upload directory: {0}")]
    InaccessibleUploadDirectory(object_store::Error),

    #[error("failed to setup the database: {0}")]
    DatabaseSetupError(#[from] DatabaseSetupError),
    
    #[error("failed to read private service key: {0}")]
    ServiceKeyReadError(std::io::Error),

    #[error("failed to read public platform key: {0}")]
    PlatformKeyReadError(std::io::Error),

    #[error("private service key could not be loaded: {0}")]
    InvalidServiceKey(jwt_simple::Error),

    #[error("public platform key could not be loaded: {0}")]
    InvalidPlatformKey(jwt_simple::Error),
}

fn load_service_key(path: &PathBuf) -> Result<SigningKey, StateSetupError> {
    let key_bytes = std::fs::read(path).map_err(StateSetupError::ServiceKeyReadError)?;
    let private_pem = String::from_utf8_lossy(&key_bytes);

    let service_key_inner =
        ES384KeyPair::from_pem(&private_pem).map_err(StateSetupError::InvalidServiceKey)?;

    let fingerprint = fingerprint_key_pair(&service_key_inner);
    let service_key_inner = service_key_inner.with_key_id(&fingerprint);

    Ok(SigningKey::new(service_key_inner))
}

fn load_platform_verfication_key(path: &PathBuf) -> Result<VerificationKey, StateSetupError> {
    let key_bytes = std::fs::read(path).map_err(StateSetupError::PlatformKeyReadError)?;
    let public_pem = String::from_utf8_lossy(&key_bytes);

    let platform_verification_key_inner =
        ES384PublicKey::from_pem(&public_pem).map_err(StateSetupError::InvalidPlatformKey)?;

    let fingerprint = sha1_fingerprint_publickey(&platform_verification_key_inner);
    let platform_verification_key_inner = platform_verification_key_inner.with_key_id(&fingerprint);

    Ok(VerificationKey::new(platform_verification_key_inner))
}