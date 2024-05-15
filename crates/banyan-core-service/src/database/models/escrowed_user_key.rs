use serde::Serialize;
use time::OffsetDateTime;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EscrowedUserKey {
    pub id: String,
    pub user_id: String,
    pub public_key: String,
    pub encrypted_private_key_material: String,
    pub pass_key_salt: String,
    pub created_at: OffsetDateTime,
}
