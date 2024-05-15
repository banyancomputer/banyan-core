use serde::{Deserialize, Serialize};

use crate::database::models::EscrowedUserKey;

#[derive(Serialize, Deserialize)]
pub struct ApiEscrowedKeyMaterial {
    pub public_key: String,
    pub encrypted_private_key_material: String,
    pub pass_key_salt: String,
}

impl From<EscrowedUserKey> for ApiEscrowedKeyMaterial {
    fn from(val: EscrowedUserKey) -> Self {
        Self {
            public_key: val.public_key,
            encrypted_private_key_material: val.encrypted_private_key_material,
            pass_key_salt: val.pass_key_salt,
        }
    }
}
