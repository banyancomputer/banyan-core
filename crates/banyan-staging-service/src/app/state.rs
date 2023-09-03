use std::fmt::Write;
use std::path::PathBuf;

use jwt_simple::prelude::*;
use object_store::local::LocalFileSystem;
use sha2::Digest;

use crate::app::{Config, Error, PlatformAuthKey, PlatformVerificationKey};
use crate::database::{self, Database};

#[derive(Clone)]
pub struct State {
    database: Database,

    platform_verification_key: PlatformVerificationKey,
    platform_auth_key: PlatformAuthKey,

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

        let db_url = match config.db_url() {
            Some(du) => du.to_string(),
            None => match std::env::var("DATABASE_URL") {
                Ok(du) => du,
                Err(_) => "sqlite://:memory:".to_string(),
            },
        };

        // Configure the database instance we're going use
        let database = database::connect(&db_url).await?;

        // Parse the platform authentication key (this will be used to communicate with the
        // metadata service).
        let key_bytes =
            std::fs::read(config.platform_auth_key_path()).map_err(Error::UnreadableSessionKey)?;
        let pem = String::from_utf8_lossy(&key_bytes);
        let auth_raw = ES384KeyPair::from_pem(&pem).map_err(Error::BadAuthenticationKey)?;
        let fingerprint = fingerprint_key(&auth_raw);
        let platform_auth_key = auth_raw.with_key_id(&fingerprint);

        // Parse the public grant verification key (this will be the one coming from the platform)
        let key_bytes = std::fs::read(config.platform_verification_key_path())
            .map_err(Error::UnreadableSessionKey)?;
        let pem = String::from_utf8_lossy(&key_bytes);
        let platform_verification_key =
            ES384PublicKey::from_pem(&pem).map_err(Error::BadAuthenticationKey)?;

        Ok(Self {
            database,

            platform_verification_key: PlatformVerificationKey::new(platform_verification_key),
            platform_auth_key: PlatformAuthKey::new(config.platform_base_url(), platform_auth_key),

            upload_directory: config.upload_directory(),
        })
    }

    pub fn platform_verification_key(&self) -> PlatformVerificationKey {
        self.platform_verification_key.clone()
    }

    pub fn upload_directory(&self) -> PathBuf {
        self.upload_directory.clone()
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

    hashed_bytes.iter().fold(String::new(), |mut output, byte| {
        let _ = write!(output, "{byte:02x}");
        output
    })
}
