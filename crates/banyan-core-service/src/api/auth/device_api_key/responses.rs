use serde::Serialize;
use validify::Validify;

#[derive(Clone, Debug, Serialize, Validify)]
pub struct CreateDeviceApiKey {
    pub id: String,
    pub account_id: String,
    pub fingerprint: String,
}
