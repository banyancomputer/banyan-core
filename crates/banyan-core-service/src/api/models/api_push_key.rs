use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ApiPushKey {
    pub fingerprint: String,
    pub pem: String,
}
