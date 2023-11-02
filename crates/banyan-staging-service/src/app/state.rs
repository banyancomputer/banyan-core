use std::fmt::Write;
use std::path::PathBuf;

use jwt_simple::prelude::*;
use object_store::local::LocalFileSystem;
use sha2::Digest;
use url::Url;

use crate::app::{Config, Error, PlatformAuthKey, PlatformVerificationKey};
use crate::database::{self, Database};

#[derive(Clone)]
pub struct State {
    database: Database,
    hostname: Url,

    platform_auth_key: PlatformAuthKey,
    platform_base_url: reqwest::Url,
    platform_verification_key: PlatformVerificationKey,

    upload_directory: PathBuf,
}

impl State {
    // not implemented as a From trait so it can be async
    pub async fn from_config(config: &Config) -> Result<Self, Error> {
        // Make sure our upload directory is present and at least readable, could do a test write
        // which wouldn't be a bad idea...
        let upload_directory = config.upload_directory().clone();
        LocalFileSystem::new_with_prefix(&upload_directory)
            .map_err(Error::InaccessibleUploadDirectory)?;

        let db_url = config.db_url();
        let db_url = Url::parse(db_url).map_err(Error::InvalidDatabaseUrl)?;
        let database = database::connect(&db_url)
            .await
            .map_err(Error::DatabaseSetup)?;

        let platform_auth_key = load_or_create_platform_auth_key(config.platform_base_url(), &config.platform_auth_key_path())?;

        // Parse the public grant verification key (this will be the one coming from the platform)
        let key_bytes = std::fs::read(config.platform_verification_key_path())
            .map_err(Error::UnreadablePlatformVerificationKey)?;
        let pem = String::from_utf8_lossy(&key_bytes);
        let platform_pub_key = ES384PublicKey::from_pem(&pem).map_err(Error::BadAuthenticationKey)?;
        let platform_verification_key = PlatformVerificationKey::new(platform_pub_key);

        Ok(Self {
            database,
            hostname: config.hostname(),

            platform_auth_key,
            platform_base_url: config.platform_base_url(),
            platform_verification_key,

            upload_directory: config.upload_directory(),
        })
    }

    pub fn hostname(&self) -> Url {
        self.hostname.clone()
    }

    pub fn platform_base_url(&self) -> Url {
        self.platform_base_url.clone()
    }

    pub fn platform_verification_key(&self) -> PlatformVerificationKey {
        self.platform_verification_key.clone()
    }

    pub fn platform_auth_key(&self) -> PlatformAuthKey {
        self.platform_auth_key.clone()
    }

    pub fn upload_directory(&self) -> PathBuf {
        self.upload_directory.clone()
    }

    pub fn database(&self) -> Database {
        self.database.clone()
    }
}

impl axum::extract::FromRef<State> for Database {
    fn from_ref(state: &State) -> Self {
        state.database.clone()
    }
}

impl axum::extract::FromRef<State> for PlatformVerificationKey {
    fn from_ref(state: &State) -> Self {
        state.platform_verification_key.clone()
    }
}

impl axum::extract::FromRef<State> for PlatformAuthKey {
    fn from_ref(state: &State) -> Self {
        state.platform_auth_key.clone()
    }
}

pub fn fingerprint_key(keys: &ES384KeyPair) -> String {
    let key_pair = keys.key_pair();
    let public_key = key_pair.public_key();
    let compressed_point = public_key.as_ref().to_encoded_point(true);

    let mut hasher = sha2::Sha256::new();
    hasher.update(compressed_point);
    let hashed_bytes = hasher.finalize();

    hashed_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn load_or_create_platform_auth_key(
    platform_url: reqwest::Url,
    path: &PathBuf,
) -> Result<PlatformAuthKey, Error> {
    tracing::info!(?path, "attempting to load or create platform auth key");
    let auth_raw = if path.exists() {
        let key_bytes =
            std::fs::read(&path).map_err(Error::UnreadableSigningKey)?;
        let private_pem = String::from_utf8_lossy(&key_bytes);

        ES384KeyPair::from_pem(&private_pem).map_err(Error::BadAuthenticationKey)?
    } else {
        let new_key = ES384KeyPair::generate();
        let private_pem = new_key.to_pem().expect("fresh keys to export");

        std::fs::write(path, &private_pem).map_err(Error::PlatformAuthFailedWrite)?;

        let public_spki = new_key
            .public_key()
            .to_pem()
            .expect("fresh key to have public component");

        let mut public_path = path.clone();
        public_path.set_extension("public");
        std::fs::write(public_path, public_spki).map_err(Error::PublicKeyWriteFailed)?;

        new_key
    };

    let fingerprint = fingerprint_key(&auth_raw);
    let auth_raw = auth_raw.with_key_id(&fingerprint);

    let mut fingerprint_path = path.clone();
    fingerprint_path.set_extension("fingerprint");
    if !fingerprint_path.exists() {
        std::fs::write(fingerprint_path, fingerprint)
            .map_err(Error::FingerprintWriteFailed)?;
    }

    Ok(PlatformAuthKey::new(platform_url, auth_raw))
}
