use banyan_api_client::prelude::*;
use jsonwebtoken::EncodingKey;

//use serde::{Deserialize, Serialize};

//#[derive(Debug, Serialize)]
//struct BucketCreationRequest {
//    friendly_name: String,
//    r#type: String,
//    initial_public_key: String,
//}
//
//#[derive(Debug, Deserialize)]
//struct BucketCreationResponse {
//    id: String,
//
//    friendly_name: String,
//    r#type: String,
//
//    root_cid: Option<String>,
//}

//#[derive(Debug, Serialize)]
//struct MetadataPublishRequest {
//    data_size: usize,
//}

//#[derive(Debug, Deserialize)]
//struct MetadataPublishResponse {
//    storage: DataStorageDetails,
//}

//#[derive(Debug, Deserialize)]
//struct DataStorageDetails {
//    authorization_ticket: String,
//    hosts: Vec<String>
//}

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

    //// Create bucket POST a struct to /api/v1/buckets
    //let bucket_creation_req = BucketCreationRequest {
    //    friendly_name: "A simple test interactive bucket".to_string(),
    //    r#type: "interactive".to_string(),
    //    initial_public_key: public_pem.clone(),
    //};

    //let bucket_creation_resp: BucketCreationResponse = http_client
    //    .post("http://127.0.0.1:3001/api/v1/buckets")
    //    .bearer_auth(&expiring_jwt)
    //    .json(&bucket_creation_req)
    //    .send()
    //    .await
    //    .unwrap()
    //    .json()
    //    .await
    //    .unwrap();

    //assert_eq!(bucket_creation_req.friendly_name, bucket_creation_resp.friendly_name);
    //assert_eq!(bucket_creation_req.r#type, bucket_creation_resp.r#type);

    //// publish bucket metadata to /api/v1/buckets/{uuid]/publish
    ////  * should read and validate the key metadata to ensure expected keys are present
    ////  * should scan the car file for number of blocks contained and return that
    ////  * metadata should be in a pending state until data has been received by storage host
    ////  * will return a storage grant for the bucket/metadata with the storage host
    //let multipart_json_data = serde_json::to_string(&MetadataPublishRequest {
    //    data_size: 1_342_100,
    //}).unwrap();

    // todo: need to workaround reqwest's multipart limitations

    //let multipart_json = reqwest::multipart::Part::bytes(multipart_json_data.as_bytes().to_vec())
    //    .mime_str("application/json")
    //    .unwrap();

    //let multipart_car_data = "some random contents for the car file...";
    //let multipart_car = reqwest::multipart::Part::bytes(multipart_car_data.as_bytes().to_vec())
    //    .mime_str("application/vnd.ipld.car; version=2")
    //    .unwrap();

    //let publish_response: MetadataPublishResponse = http_client
    //    .post(format!("http://127.0.0.1:3001/api/v1/buckets/{}/publish", bucket_creation_resp.id))
    //    .bearer_auth(&expiring_jwt)
    //    .multipart(reqwest::multipart::Form::part(multipart_json))
    //    .multipart(reqwest::multipart::Form::part(multipart_car))
    //    .send()
    //    .await
    //    .unwrap()
    //    .json()
    //    .await
    //    .unwrap();
}
