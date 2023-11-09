use serde::{Deserialize, Serialize};

use crate::database::models::EscrowedDevice;

#[derive(Serialize, Deserialize)]
pub struct ApiEscrowedKeyMaterial {
    pub api_public_key_pem: String,
    pub encryption_public_key_pem: String,
    pub encrypted_private_key_material: String,
    pub pass_key_salt: String,
}

impl From<EscrowedDevice> for ApiEscrowedKeyMaterial {
    fn from(val: EscrowedDevice) -> Self {
        Self {
            api_public_key_pem: val.api_public_key_pem,
            encryption_public_key_pem: val.encryption_public_key_pem,
            encrypted_private_key_material: val.encrypted_private_key_material,
            pass_key_salt: val.pass_key_salt,
        }
    }
}
