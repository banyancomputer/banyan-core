use std::path::PathBuf;

use banyan_object_store::{
    ObjectStore, ObjectStoreConnection, ObjectStoreConnectionError, ObjectStoreError,
};
use jwt_simple::prelude::*;
use url::Url;

use crate::app::{Config, Secrets};
use crate::database::{self, Database, DatabaseSetupError};
use crate::utils::{fingerprint_key_pair, fingerprint_public_key, SigningKey, VerificationKey};

#[derive(Clone)]
pub struct State {
    // Resources
    /// Access to the database
    database: Database,
    /// Connection configuration for the upload store
    upload_store_connection: ObjectStoreConnection,

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
        // Try and parse the upload store connection
        let upload_store_connection = config.upload_store_url().try_into()?;
        // Do a test setup to make sure the upload store exists and is writable as an early
        // sanity check
        ObjectStore::new(&upload_store_connection)?;

        let database = database::connect(&config.database_url()).await?;

        let service_signing_key = load_or_create_service_key(&config.service_key_path())?;
        let service_verification_key = service_signing_key.verifier();

        let platform_verification_key =
            load_platform_verfication_key(&config.platform_public_key_path())?;

        let secrets = Secrets::new(service_signing_key);

        Ok(Self {
            database,
            upload_store_connection,

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

    pub fn upload_store_connection(&self) -> &ObjectStoreConnection {
        &self.upload_store_connection
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

#[derive(Debug, thiserror::Error)]
pub enum StateSetupError {
    #[error("unable to parse url for upload store: {0}")]
    ObjectStoreConnection(#[from] ObjectStoreConnectionError),

    #[error("unable to access upload store: {0}")]
    ObjectStore(#[from] ObjectStoreError),

    #[error("failed to setup the database: {0}")]
    DatabaseSetup(#[from] DatabaseSetupError),

    #[error("failed to read private service key: {0}")]
    ServiceKeyRead(std::io::Error),

    #[error("failed to write service key: {0}")]
    ServiceKeyWriteFailed(std::io::Error),

    #[error("failed to read public platform key: {0}")]
    PlatformKeyRead(std::io::Error),

    #[error("private service key could not be loaded: {0}")]
    InvalidServiceKey(jwt_simple::Error),

    #[error("public platform key could not be loaded: {0}")]
    InvalidPlatformKey(jwt_simple::Error),
}

fn load_or_create_service_key(path: &PathBuf) -> Result<SigningKey, StateSetupError> {
    // Try to load or otherwise generate a new key
    let service_key_inner = if path.exists() {
        let service_key_bytes = std::fs::read(path).map_err(StateSetupError::ServiceKeyRead)?;
        let service_key_pem = String::from_utf8_lossy(&service_key_bytes);
        let service_key =
            ES384KeyPair::from_pem(&service_key_pem).map_err(StateSetupError::InvalidServiceKey)?;

        let fingerprint = fingerprint_key_pair(&service_key);

        service_key.with_key_id(&fingerprint)
    } else {
        let service_key = ES384KeyPair::generate();

        // Write out the private key
        let service_key_pem = service_key.to_pem().expect("key to export");
        std::fs::write(path, service_key_pem).map_err(StateSetupError::ServiceKeyWriteFailed)?;

        // Write out the public key
        let mut path = path.clone();
        path.set_extension("public");
        let service_public_key_pem = service_key.public_key().to_pem().expect("key to export");
        std::fs::write(path.clone(), service_public_key_pem)
            .map_err(StateSetupError::ServiceKeyWriteFailed)?;

        // Write out the fingerprint
        let mut path = path.clone();
        path.set_extension("fingerprint");
        let fingerprint = fingerprint_key_pair(&service_key);
        std::fs::write(path, &fingerprint).map_err(StateSetupError::ServiceKeyWriteFailed)?;

        service_key.with_key_id(&fingerprint)
    };

    Ok(SigningKey::new(service_key_inner))
}

fn load_platform_verfication_key(path: &PathBuf) -> Result<VerificationKey, StateSetupError> {
    let key_bytes = std::fs::read(path).map_err(StateSetupError::PlatformKeyRead)?;
    let public_pem = String::from_utf8_lossy(&key_bytes);

    let platform_verification_key_inner =
        ES384PublicKey::from_pem(&public_pem).map_err(StateSetupError::InvalidPlatformKey)?;

    // TODO: use normalized fingerprint -- blake3
    let fingerprint = fingerprint_public_key(&platform_verification_key_inner);
    let platform_verification_key_inner = platform_verification_key_inner.with_key_id(&fingerprint);

    Ok(VerificationKey::new(platform_verification_key_inner))
}

#[cfg(test)]
pub mod test {
    use axum::extract::State;
    use banyan_object_store::ObjectStoreConnection;
    use jwt_simple::algorithms::ES384KeyPair;
    use url::Url;

    use crate::app::{AppState, Secrets};
    use crate::database::Database;
    use crate::utils::{SigningKey, VerificationKey};

    pub fn mock_app_state(database: Database) -> State<AppState> {
        let platform_key = ES384KeyPair::generate();
        let platform_public_key = platform_key.public_key();
        let service_key = ES384KeyPair::generate();
        let service_public_key = service_key.public_key();
        let url = Url::parse("file:///tmp").unwrap();

        State(AppState {
            database,
            upload_store_connection: ObjectStoreConnection::try_from(url).unwrap(),
            secrets: Secrets::new(SigningKey::new(platform_key)),
            service_name: "service_name".to_string(),
            service_hostname: Url::parse("http://127.0.0.1:3001").unwrap(),
            service_verification_key: VerificationKey::new(service_public_key),
            platform_name: "platform_name".to_string(),
            platform_hostname: Url::parse("http://127.0.0.1:3002").unwrap(),
            platform_verification_key: VerificationKey::new(platform_public_key),
        })
    }
}
