use std::path::PathBuf;

use banyan_api_client::prelude::*;
use jsonwebtoken::EncodingKey;

#[tokio::main]
async fn main() {
    let mut api_client = ClientBuilder::new().build().unwrap();

    let account_info = api_client
        .call(banyan_api_client::fake::RegisterFakeAccount)
        .await
        .unwrap();

    let private_pem = banyan_api_client::fake::create_private_ec_pem();
    let public_pem = banyan_api_client::fake::public_from_private(&private_pem);

    let device_key_info = api_client
        .call(banyan_api_client::fake::FakeRegisterDeviceKey {
            token: account_info.token,
            public_key: public_pem.clone(),
        })
        .await
        .unwrap();

    let fingerprint = banyan_api_client::fake::fingerprint_public_pem(public_pem.as_str());

    assert_eq!(account_info.id, device_key_info.account_id);
    assert_eq!(fingerprint, device_key_info.fingerprint);

    let jwt_signing_key = EncodingKey::from_ec_pem(private_pem.as_bytes()).unwrap();
    api_client.set_credentials(account_info.id, fingerprint, jwt_signing_key);

    let authenticated_info = api_client
        .call(WhoAmI)
        .await
        .unwrap();

    println!("{authenticated_info:?}");

    let bucket_info = api_client.call(CreateBucket {
            friendly_name: "Testing Interactive Bucket".to_string(),
            r#type: BucketType::Interactive,
            initial_public_key: "ECDH public key pem formatted bits".to_string(),
        })
        .await
        .unwrap();

    println!("{bucket_info:?}");

    let publish_details = api_client.call(PublishBucketMetadata {
            bucket_id: bucket_info.id.clone(),
            metadata_path: PathBuf::from("./path/to/file.car"),
            expected_data_size: 1_567_129,
        })
        .await
        .unwrap();

    println!("{publish_details:?}");
}
