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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_cid: Option<String>,
    pub public_keys: Vec<PublicKeySummary>,
}

#[derive(Serialize)]
pub struct MinimalBucket {
    pub id: String,

    pub friendly_name: String,
    pub r#type: BucketType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_cid: Option<String>,
}

#[derive(Serialize)]
pub struct PublicKeySummary {
    pub approved: bool,
    pub fingerprint: String,
    pub pem: String,
}
