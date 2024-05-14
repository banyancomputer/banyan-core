use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ApiPushKey {
    pub fingerprint: String,
    pub public_key: String,
}
