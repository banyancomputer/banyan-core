mod create;
mod delete;
mod get;
mod list;

pub use create::CreateBucket;
pub use delete::DeleteBucket;
pub use get::GetBucket;
pub use list::ListBuckets;

use serde::Deserialize;
use uuid::Uuid;
use crate::BucketType;

#[derive(Debug, Deserialize, PartialEq)]
pub struct BucketInfoResponse {
    pub id: Uuid,
    pub friendly_name: String,
    pub r#type: BucketType,
}

#[cfg(test)]
mod test {
    use crate::{prelude::{bucket::*, test::fake_authenticated_client}, client::ClientError, BucketType, PublishBucketMetadata};
    
    #[tokio::test]
    async fn create() -> Result<(), ClientError> {
        let mut api_client = fake_authenticated_client().await;
        let friendly_name = "test interactive bucket".to_string();
        let response = api_client.call(CreateBucket {
            friendly_name: friendly_name.clone(),
            r#type: BucketType::Interactive,
            initial_public_key: "ECDH public key pem formatted bits".to_string(),
        }).await?;

        assert_eq!(friendly_name, response.friendly_name);
        assert_eq!(BucketType::Interactive, response.r#type);

        Ok(())
    }

    #[tokio::test]
    async fn create_get() -> Result<(), ClientError> {
        let mut api_client = fake_authenticated_client().await;
        let friendly_name = "test interactive bucket".to_string();
        let response1 = api_client.call(CreateBucket {
            friendly_name: friendly_name.clone(),
            r#type: BucketType::Interactive,
            initial_public_key: "ECDH public key pem formatted bits".to_string(),
        }).await?;

        let response2 = api_client.call(GetBucket {
            bucket_id: response1.id,
        }).await?;

        assert_eq!(response1, response2);
        Ok(())
    }

    #[tokio::test]
    async fn list() -> Result<(), ClientError> {
        let mut api_client = fake_authenticated_client().await;
        let buckets = api_client.call(ListBuckets{

        }).await?;

        println!("{buckets:?}");

        Ok(())
    }

    #[tokio::test]
    async fn main_test() -> Result<(), ClientError> {
        let mut api_client = fake_authenticated_client().await;

        // Create a new interactive bucket
        let bucket_info = api_client
            .call(CreateBucket {
                friendly_name: "Testing Interactive Bucket".to_string(),
                r#type: BucketType::Interactive,
                initial_public_key: "ECDH public key pem formatted bits".to_string(),
            })
            .await?;
        println!("{bucket_info:?}");

        // Publish a metadata file to the bucket we just created

        // Can be anything that can be turned into a streaming reqwest::Body including file IO,
        // network, etc. These chunks are simulating a TryStream as a static fixture. All the pieces
        // will be consumed one by one and be present in the request directly.
        let chunks: Vec<Result<_, ::std::io::Error>> = vec![
            Ok("PRAGMA BITS\n"),
            Ok("Some other car things\n"),
            Ok("and sure lets throw in an index"),
        ];
        let raw_stream = futures::stream::iter(chunks);
        let metadata_stream = reqwest::Body::wrap_stream(raw_stream);

        let publish_details = api_client
            .call(PublishBucketMetadata {
                bucket_id: bucket_info.id,
                expected_data_size: 1_567_129,
                metadata_cid: "a real CID I promise!".to_string(),
                root_cid: "rooty McCIDFace".to_string(),

                metadata_stream,
            })
            .await?;

        println!("{publish_details:?}");

        Ok(())
    }

}