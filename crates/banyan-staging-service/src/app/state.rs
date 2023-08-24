use std::sync::Arc;

use jwt_simple::algorithms::{ES384KeyPair, ECDSAP384KeyPairLike};
use object_store::local::LocalFileSystem;
use sha2::Digest;

use crate::app::{Config, Error};
use crate::database::{config_database, Db};

#[derive(Clone)]
pub struct State {
    database: Db,
    jwt_key: Arc<ES384KeyPair>,
}

impl State {
    // not implemented as a From trait so it can be async
    pub async fn from_config(config: &Config) -> Result<Self, Error> {
        let upload_directory = config.upload_directory().clone();
        LocalFileSystem::new_with_prefix(&upload_directory)
            .map_err(Error::inaccessible_upload_directory)?;

        let database = config_database(&config).await?;

        let key_bytes = std::fs::read(config.jwt_key_path()).map_err(Error::unreadable_key)?;
        let pem = String::from_utf8_lossy(&key_bytes);
        let mut jwt_key_raw = ES384KeyPair::from_pem(&pem).map_err(Error::invalid_key)?;

        let fingerprint = fingerprint_jwt_key(&jwt_key_raw);
        jwt_key_raw = jwt_key_raw.with_key_id(&fingerprint);
        let jwt_key = Arc::new(jwt_key_raw);

        Ok(Self { database, jwt_key })
    }
}

impl axum::extract::FromRef<State> for Db {
    fn from_ref(state: &State) -> Self {
        state.database.clone()
    }
}

impl axum::extract::FromRef<State> for Arc<ES384KeyPair> {
    fn from_ref(state: &State) -> Self {
        state.jwt_key.clone()
    }
}

fn fingerprint_jwt_key(jwt_keys: &ES384KeyPair) -> String {
    let public_key = jwt_keys.key_pair().public_key();
    let compressed_point = public_key.as_ref().to_encoded_point(true);

    let mut hasher = sha2::Sha256::new();
    hasher.update(compressed_point);
    let hashed_bytes = hasher.finalize();

    hashed_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
