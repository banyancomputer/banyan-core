use serde::Serialize;
use time::OffsetDateTime;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EscrowedDevice {
    pub id: String,
    pub user_id: String,
    pub api_public_key_pem: String,
    pub encryption_public_key_pem: String,
    pub encrypted_private_key_material: String,
    pub pass_key_salt: String,
    pub created_at: OffsetDateTime,
}