use jsonwebtoken::EncodingKey;
use openssl::bn::BigNumContext;
use openssl::ec::{EcGroup, EcKey, PointConversionForm};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private, Public};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct AccountCreationResponse {
    id: String,
    token: String,
}

#[derive(Debug, Serialize)]
struct DeviceKeyRegistrationRequest {
    public_key: String,
}

#[derive(Debug, Deserialize)]
struct DeviceKeyRegistrationResponse {
    id: String,
    account_id: String,
    fingerprint: String,
}

#[tokio::main]
async fn main() {
    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.insert("Content-Type", reqwest::header::HeaderValue::from_static("application/json"));

    let http_client = reqwest::Client::builder()
        .default_headers(default_headers)
        .user_agent("banyan-test-client/v0.1")
        .build()
        .unwrap();

    // get fake account (get /api/v1/auth/fake_register)
    //  * will get us account id and account token
    let fake_account_response: AccountCreationResponse = http_client
        .get("http://127.0.0.1:3000/api/v1/auth/fake_register")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // create client/device ec keys

    // Get us just the private client/device key we want to be able to get anything we need just
    // from this.
    let private_key: PKey<Private> = {
        let ec_group = EcGroup::from_curve_name(Nid::SECP384R1).unwrap();
        let ec_key = EcKey::generate(&ec_group).unwrap();
        ec_key.try_into().unwrap()
    };

    // Get the public key so we can calculate the fingerprint
    let public_key: PKey<Public> = {
        let ec_group = EcGroup::from_curve_name(Nid::SECP384R1).unwrap();
        let priv_ec_key = private_key.ec_key().unwrap();
        let pub_ec_key: EcKey<Public> = EcKey::from_public_key(&ec_group, priv_ec_key.public_key()).unwrap();

        PKey::from_ec_key(pub_ec_key).unwrap()
    };

    // Calculate our fingerprint
    let fingerprint: String = {
        let ec_group = EcGroup::from_curve_name(Nid::SECP384R1).unwrap();
        let mut big_num_ctx = BigNumContext::new().unwrap();

        let ec_pub_key = public_key.ec_key().unwrap();
        let pub_key_bytes = ec_pub_key
            .public_key()
            .to_bytes(
                &ec_group,
                PointConversionForm::COMPRESSED,
                &mut big_num_ctx,
            )
            .unwrap();

        let fingerprint_bytes = openssl::sha::sha1(&pub_key_bytes);

        fingerprint_bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<String>>()
            .join(":")
    };

    // We need the private pem bytes for use with jsonwebtoken's EncodingKey
    let private_pem_bytes = private_key.private_key_to_pem_pkcs8().unwrap();

    // We need the public pem bytes to register with the API
    let public_pem_bytes = private_key.public_key_to_pem().unwrap();

    // Create an encoding key with the private key
    let jwt_signing_key = EncodingKey::from_ec_pem(private_pem_bytes.as_ref()).unwrap();

    let bearer_val = format!("Bearer {}", fake_account_response.token);
    let temp_account_bearer_header = reqwest::header::HeaderValue::from_str(&bearer_val).unwrap();

    let device_reg_req = DeviceKeyRegistrationRequest {
        public_key: String::from_utf8_lossy(&public_pem_bytes).to_string(),
    };

    // register client/device ec keys POST a struct to /api/v1/auth/register_device_key
    //  * uses account token as bearer token in authorization header
    let device_key_reg_response: DeviceKeyRegistrationResponse = http_client
        .post("http://127.0.0.1:3000/api/v1/auth/register_device_key")
        .header("Authorization", temp_account_bearer_header)
        .json(&device_reg_req)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    println!("{device_key_reg_response:?}");

    // create bucket POST a struct to /api/v1/buckets
    //  * uses a signed bearer token from the client/device key
    //  * will get use the bucket uuid

    // publish bucket metadata to /api/v1/buckets/{uuid]/publish
    //  * needs to be switched to multipart, we need to post a struct with this as well with the
    //    size of the data storage needed
    //  * should want content type "application/vnd.ipld.car; version=2"
    //  * should read and validate the key metadata to ensure expected keys are present
    //  * should scan the car file for number of blocks contained and return that
    //  * metadata should be in a pending state until data has been received by storage host
    //  * will return a storage grant for the bucket/metadata with the storage host
}
