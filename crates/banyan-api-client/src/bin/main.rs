use banyan_api_client::prelude::*;

//use jsonwebtoken::{get_current_timestamp, Algorithm, Header, EncodingKey};
//use openssl::bn::BigNumContext;
//use openssl::ec::{EcGroup, EcKey, PointConversionForm};
//use openssl::nid::Nid;
//use openssl::pkey::{PKey, Private, Public};
//use serde::{Deserialize, Serialize};

//#[derive(Debug, Deserialize)]
//struct AccountCreationResponse {
//    id: String,
//    token: String,
//}
//
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
//
//#[derive(Debug, Serialize)]
//struct DeviceKeyRegistrationRequest {
//    public_key: String,
//}
//
//#[derive(Debug, Deserialize)]
//struct DeviceKeyRegistrationResponse {
//    id: String,
//    account_id: String,
//    fingerprint: String,
//}
//
//#[derive(Debug, Serialize)]
//struct MetadataPublishRequest {
//    data_size: usize,
//}
//
//#[derive(Debug, Deserialize)]
//struct MetadataPublishResponse {
//    storage: DataStorageDetails,
//}
//
//#[derive(Debug, Deserialize)]
//struct DataStorageDetails {
//    authorization_ticket: String,
//    hosts: Vec<String>
//}

#[tokio::main]
async fn main() {
    let mut api_client = ClientBuilder::new().build().unwrap();

    let response = api_client.call(banyan_api_client::fake::RegisterFakeAccount).await.unwrap();
    println!("{response:?}");

    // create client/device ec keys

    // Get us just the private client/device key we want to be able to get anything we need just
    // from this.
    //let private_key: PKey<Private> = {
    //    let ec_group = EcGroup::from_curve_name(Nid::SECP384R1).unwrap();
    //    let ec_key = EcKey::generate(&ec_group).unwrap();
    //    ec_key.try_into().unwrap()
    //};

    //// Get the public key so we can calculate the fingerprint
    //let public_key: PKey<Public> = {
    //    let ec_group = EcGroup::from_curve_name(Nid::SECP384R1).unwrap();
    //    let priv_ec_key = private_key.ec_key().unwrap();
    //    let pub_ec_key: EcKey<Public> = EcKey::from_public_key(&ec_group, priv_ec_key.public_key()).unwrap();

    //    PKey::from_ec_key(pub_ec_key).unwrap()
    //};

    //// Calculate our fingerprint
    //let fingerprint: String = {
    //    let ec_group = EcGroup::from_curve_name(Nid::SECP384R1).unwrap();
    //    let mut big_num_ctx = BigNumContext::new().unwrap();

    //    let ec_pub_key = public_key.ec_key().unwrap();
    //    let pub_key_bytes = ec_pub_key
    //        .public_key()
    //        .to_bytes(
    //            &ec_group,
    //            PointConversionForm::COMPRESSED,
    //            &mut big_num_ctx,
    //        )
    //        .unwrap();

    //    let fingerprint_bytes = openssl::sha::sha1(&pub_key_bytes);

    //    fingerprint_bytes
    //        .iter()
    //        .map(|byte| format!("{byte:02x}"))
    //        .collect::<Vec<String>>()
    //        .join(":")
    //};

    //// We need the public pem bytes to register with the API
    //let public_pem = String::from_utf8_lossy(&private_key.public_key_to_pem().unwrap()).to_string();

    //let device_reg_req = DeviceKeyRegistrationRequest {
    //    public_key: public_pem.clone(),
    //};

    //// register client/device ec keys POST a struct to /api/v1/auth/register_device_key
    ////  * uses account token as bearer token in authorization header
    //let device_raw_response = http_client
    //    .post("http://127.0.0.1:3001/api/v1/auth/register_device_key")
    //    .bearer_auth(&fake_account_response.token)
    //    .json(&device_reg_req)
    //    .send()
    //    .await
    //    .unwrap();

    //let device_key_reg_response: DeviceKeyRegistrationResponse = device_raw_response
    //    .json()
    //    .await
    //    .unwrap();

    //let device_key_reg_response: DeviceKeyRegistrationResponse = http_client
    //    .post("http://127.0.0.1:3001/api/v1/auth/register_device_key")
    //    .bearer_auth(&fake_account_response.token)
    //    .json(&device_reg_req)
    //    .send()
    //    .await
    //    .unwrap()
    //    .json()
    //    .await
    //    .unwrap();

    //assert_eq!(fake_account_response.id, device_key_reg_response.account_id);
    //assert_eq!(fingerprint, device_key_reg_response.fingerprint);

    //// We need the private pem bytes for use with jsonwebtoken's EncodingKey
    //let private_pem = String::from_utf8_lossy(&private_key.private_key_to_pem_pkcs8().unwrap()).to_string();
    //// Create an encoding key with the private key
    //let jwt_signing_key = EncodingKey::from_ec_pem(private_pem.as_bytes()).unwrap();

    // From here on out we should be using the client instead of our jank bits

    //let expiring_jwt = {
    //    let api_token = ApiToken {
    //        // todo: generate random string here
    //        nonce: None,

    //        audience: "banyan-platform".to_string(),
    //        subject: fake_account_response.id.clone(),

    //        expiration: get_current_timestamp() + 870,
    //        not_before: get_current_timestamp() - 30,
    //    };

    //    let bearer_header = Header {
    //        alg: Algorithm::ES384,
    //        kid: Some(fingerprint.clone()),
    //        ..Default::default()
    //    };

    //    jsonwebtoken::encode(&bearer_header, &api_token, &jwt_signing_key).unwrap()
    //};

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

    //println!("{publish_response:?}");
}
