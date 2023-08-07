use serde::Serialize;

#[derive(Serialize)]
pub struct NewAccount {
    pub id: String,
    pub token: String,
}

#[derive(Serialize)]
pub struct NewDeviceKey {
    pub id: String,
    pub account_id: String,
    pub fingerprint: String,
}

#[derive(Serialize)]
pub struct WhoAmI {
    pub account_id: String,
}
