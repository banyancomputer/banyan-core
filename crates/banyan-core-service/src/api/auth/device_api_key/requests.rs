use serde::Deserialize;
use validify::Validify;

#[derive(Clone, Debug, Deserialize, Validify)]
pub struct CreateDeviceApiKey {
    pem: String,
}

impl CreateDeviceApiKey {
    pub fn pem(&self) -> &str {
        self.pem.as_str()
    }
}
