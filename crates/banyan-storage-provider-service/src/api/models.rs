use serde::Deserialize;

#[derive(Deserialize)]
pub struct ApiClients {
    pub platform_id: String,
    pub fingerprint: String,
    pub public_key: String,
}
