use serde::Serialize;
use validify::Validify;

#[derive(Clone, Debug, Serialize, Validify)]
pub struct CreateDeviceApiKey {
    pub id: String,
    pub account_id: String,
    pub fingerprint: String,
}

#[derive(Clone, Debug, Serialize, Validify)]
pub struct ReadDeviceApiKey {
    pub id: String,
    pub account_id: String,
    pub fingerprint: String,
    pub pem: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReadAllDeviceApiKeys(pub Vec<ReadDeviceApiKey>); 

#[derive(Clone, Debug, Serialize, Validify)]
pub struct DeleteDeviceApiKey {
    pub id: String,
    pub account_id: String,
    pub fingerprint: String,
}
