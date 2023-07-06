use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BucketType {
    Backup,
    Interactive,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Client {
    Web,
    Api { friendly_name: String, id: Uuid },
}

#[derive(Serialize)]
pub struct DetailedBucket {
    pub uuid: Uuid,
    pub friendly_name: String,
    pub r#type: BucketType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data_cid: Option<String>,
    pub public_keys: Vec<PublicKeySummary>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct MinimalBucket {
    pub uuid: Uuid,

    pub friendly_name: String,
    pub r#type: BucketType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data_cid: Option<String>,

    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ProtectedKey(pub String);

#[derive(Serialize)]
pub struct PublicKeySummary {
    pub client: Client,
    pub fingerprint: String,
    pub status: PublicKeyStatus,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicKeyStatus {
    Approved(ProtectedKey),
    Pending,
}
