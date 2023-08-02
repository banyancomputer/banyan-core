use serde::Deserialize;
use validify::Validify;

#[derive(Clone, Debug, Deserialize, Validify)]
pub struct RegisterDeviceKey {
    public_key: String,
}

impl RegisterDeviceKey {
    pub fn public_key(&self) -> &str {
        self.public_key.as_str()
    }
}
