use serde::{Deserialize, Serialize};
use validify::Validify;

#[derive(Clone, Debug, Serialize, Deserialize, Validify)]
pub struct CreateFakeAccount {
    device_api_key_pem: String,
}

impl CreateFakeAccount {
    pub fn device_api_key_pem(&self) -> &str {
        self.device_api_key_pem.as_str()
    }
}
