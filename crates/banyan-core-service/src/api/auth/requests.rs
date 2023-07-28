use serde::Deserialize;
use validify::Validify;

#[derive(Clone, Debug, Deserialize, Validify)]
/// Format for requests to register a Device
/// public_ecdsa_key_pem - the device's write public  (PEM formatted)
/// public_ecdh_key_pem - the device's exchange key (PEM formatted)
pub struct RegisterDeviceKey {
    public_key: String,
}


impl RegisterDeviceKey {
    pub fn public_key(&self) -> &str {
        self.public_key.as_str()
    }
}
