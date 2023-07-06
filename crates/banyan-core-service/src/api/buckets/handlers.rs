use axum::extract::{self, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::{DateTime, FixedOffset, Utc};
use uuid::Uuid;
use validify::Validate;

use crate::api::buckets::requests::*;
use crate::api::buckets::responses::*;

pub async fn create(extract::Json(new_bucket): extract::Json<CreateBucket>) -> impl IntoResponse {
    if let Err(errors) = new_bucket.validate() {
        return (
            StatusCode::BAD_REQUEST,
            format!("errors: {:?}", errors.errors()),
        );
    }

    (StatusCode::OK, "created".to_string())
}

pub async fn index() -> impl IntoResponse {
    let bucket_list = vec![
        MinimalBucket {
            uuid: Uuid::parse_str("79bfee96-0a93-4f79-87d1-212675823d6a").expect("valid uuid"),
            friendly_name: "test interactive bucket".to_string(),
            r#type: BucketType::Interactive,
            meta_data_cid: Some(
                "bafybeihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku".to_string(),
            ),
            updated_at: DateTime::<Utc>::from(
                DateTime::<FixedOffset>::parse_from_rfc3339("2023-07-03T16:39:57-00:00")
                    .expect("valid format"),
            ),
        },
        MinimalBucket {
            uuid: Uuid::parse_str("7bce1c56-71b9-4147-80d4-7519a7e98bd3").expect("valid uuid"),
            friendly_name: "test backup bucket".to_string(),
            r#type: BucketType::Backup,
            meta_data_cid: Some("QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n".to_string()),
            updated_at: DateTime::<Utc>::from(
                DateTime::<FixedOffset>::parse_from_rfc3339("2023-06-21T10:51:58-00:00")
                    .expect("valid format"),
            ),
        },
    ];

    (StatusCode::OK, axum::Json(bucket_list))
}

pub async fn show(Path(bucket_id): Path<Uuid>) -> impl IntoResponse {
    let bucket = DetailedBucket {
        uuid: bucket_id,
        friendly_name: "test interactive bucket".to_string(),
        r#type: BucketType::Interactive,

        meta_data_cid: Some(
            "bafybeihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku".to_string(),
        ),
        public_keys: vec![
            PublicKeySummary {
                client: Client::Web,
                fingerprint: "0b:9e:89:30:d9:3d:36:17:f6:ca:43:ad:bf:b7:8f:32:97:40:39:f2"
                    .to_string(),
                status: PublicKeyStatus::Approved(ProtectedKey(
                    "YSBzZWNyZXQga2V5IGVuY3J5cHRlZCB3aXRoIGEgcHVibGljIGtleQo=".to_string(),
                )),
            },
            PublicKeySummary {
                client: Client::Api {
                    friendly_name: "My Laptop API Client Key".to_string(),
                    id: Uuid::parse_str("f412b1c8-14ec-41fc-87b9-42d9e6e7429a")
                        .expect("valid uuid"),
                },
                fingerprint: "a3:b5:9e:5f:e8:84:ee:1f:34:d9:8e:ef:85:8e:3f:b6:62:ac:10:4a"
                    .to_string(),
                status: PublicKeyStatus::Pending,
            },
        ],

        created_at: DateTime::<Utc>::from(
            DateTime::<FixedOffset>::parse_from_rfc3339("2023-03-21T01:23:24-00:00")
                .expect("valid format"),
        ),
        updated_at: DateTime::<Utc>::from(
            DateTime::<FixedOffset>::parse_from_rfc3339("2023-07-03T16:39:57-00:00")
                .expect("valid format"),
        ),
    };

    (StatusCode::OK, axum::Json(bucket))
}
