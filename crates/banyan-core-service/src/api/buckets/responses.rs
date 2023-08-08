use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "lowercase")]
pub enum BucketType {
    Backup,
    Interactive,
}

#[derive(Serialize)]
pub struct DetailedBucket {
    pub id: String,
    pub friendly_name: String,
    pub r#type: BucketType,

    pub public_keys: Vec<PublicKeySummary>,
}

#[derive(Serialize)]
pub struct MinimalBucket {
    pub id: String,

    pub friendly_name: String,
    pub r#type: BucketType,
}

#[derive(Serialize)]
pub struct PublicKeySummary {
    pub approved: bool,
    pub fingerprint: String,
    pub pem: String,
}

#[derive(Debug, Serialize)]
pub struct PublishBucketMetadataResponse {
    pub id: String,
    pub state: MetadataState,

    pub storage_host: String,
    pub storage_authorization: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataState {
    Pending,
}
