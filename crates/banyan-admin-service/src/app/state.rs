use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use banyan_api_client::prelude::{Client, ClientBuilder};
use jsonwebtoken::EncodingKey;
use jwt_simple::prelude::*;
use url::Url;
use uuid::Uuid;

use crate::app::config::Config;
use crate::app::secrets::{ProviderCredential, Secrets, ServiceKey};
use crate::app::ServiceVerificationKey;
use crate::database::{connect, Database, DatabaseSetupError};
use crate::utils::fingerprint_key_pair;

#[derive(Clone)]
pub struct State {
    database: Database,
    secrets: Secrets,
    service_name: String,
    service_hostname: Url,
    service_verification_key: ServiceVerificationKey,
    client: Client,
}

impl State {
    pub async fn from_config(config: &Config) -> Result<Self, StateSetupError> {
        // Try and parse the upload store connection
        // Do a test setup to make sure the upload store exists and is writable as an early
        // sanity check
        let database = connect(&config.database_url()).await?;

        let service_signing_key = load_or_create_service_key(&config.service_key_path())?;
        let service_verification_key = service_signing_key.verifier();
        let banyan_admin_id = Uuid::parse_str("1538defd-2375-490b-b1eb-54a10b57153b").unwrap();
        let mut credentials = BTreeMap::new();
        credentials.insert(
            Arc::from("google"),
            ProviderCredential::new(config.google_client_id(), config.google_client_secret()),
        );
        let encoding_key =
            EncodingKey::from_ec_pem(service_signing_key.to_pem().unwrap().as_bytes()).unwrap();
        let secrets = Secrets::new(credentials, service_signing_key);
        let mut api_client = ClientBuilder::default().build().expect("client");
        api_client.set_credentials(banyan_admin_id, banyan_admin_id.to_string(), encoding_key);

        Ok(Self {
            database,
            client: api_client,

            secrets,

            service_name: config.service_name().to_string(),
            service_hostname: config.service_hostname().clone(),
            service_verification_key,
        })
    }

    pub fn database(&self) -> Database {
        self.database.clone()
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

    pub fn service_verification_key(&self) -> ServiceVerificationKey {
        self.service_verification_key.clone()
    }
    pub fn client(&self) -> Client {
        self.client.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StateSetupError {
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

fn load_or_create_service_key(path: &PathBuf) -> Result<ServiceKey, StateSetupError> {
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

    Ok(ServiceKey::new(service_key_inner))
}
