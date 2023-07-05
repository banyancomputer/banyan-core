use axum::extract::Path;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::api::buckets::responses::*;

pub async fn show(Path(bucket_id): Path<Uuid>) -> impl IntoResponse {
    Bucket {
        uuid: bucket_id,
        friendly_name: "test bucket".to_string(),
        r#type: BucketType::Interactive,

        meta_data_cid: "some cid like thing".to_string(),
        public_keys: vec![
            PublicKey {
                client: Client::Web,
                fingerprint: "0b:9e:89:30:d9:3d:36:17:f6:ca:43:ad:bf:b7:8f:32:97:40:39:f2".to_string(),
                status: PublicKeyStatus::Approved(ProtectedKey("YSBzZWNyZXQga2V5IGVuY3J5cHRlZCB3aXRoIGEgcHVibGljIGtleQo=".to_string())),
            },
            PublicKey {
                client: Client::Api {
                    friendly_name: "My Laptop API Client Key".to_string(),
                    id: Uuid::parse_str("f412b1c8-14ec-41fc-87b9-42d9e6e7429a").expect("valid uuid"),
                },
                fingerprint: "a3:b5:9e:5f:e8:84:ee:1f:34:d9:8e:ef:85:8e:3f:b6:62:ac:10:4a".to_string(),
                status: PublicKeyStatus::Pending,
            },
        ],
    }
}
