use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct ApiEscrowedKeyMaterial {
    pub api_public_key_pem: String,
    pub encryption_public_key_pem: String,
    pub encrypted_private_key_material: String,
    pub pass_key_salt: String,
}