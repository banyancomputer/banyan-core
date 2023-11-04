use serde::{Serialize, Deserialize};

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct EscrowedKeyMaterial {
    pub api_public_key_pem: String,
    pub encryption_public_key_pem: String,
    pub encrypted_private_key_material: String,
    pub pass_key_salt: String,
}

impl EscrowedKeyMaterial {
    pub fn api_public_key_pem(&self) -> String {
        self.api_public_key_pem.clone()
    }
    pub fn encryption_public_key_pem(&self) -> String {
        self.encryption_public_key_pem.clone()
    }
    pub fn encrypted_private_key_material(&self) -> String {
        self.encrypted_private_key_material.clone()
    }
    pub fn pass_key_salt(&self) -> String {
        self.pass_key_salt.clone()
    }
}
