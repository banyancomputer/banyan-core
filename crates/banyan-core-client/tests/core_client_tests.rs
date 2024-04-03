use std::fmt::{Debug, Display};
use banyan_core_client::apis::buckets_api::get_single_bucket;
use banyan_core_client::apis::configuration::Configuration;
use banyan_core_client::apis::Error;

#[tokio::test]
async fn test_example_call() {
    let mut config  = Configuration::new();
    config.bearer_access_token = Some("the-signed-jwt".to_string());
    let bucket_id = "123";
    let bucket_res = get_single_bucket(&config,bucket_id).await;
    match bucket_res {
        Ok(e) => {
            println!("{}", e.id);
        },
        Err(e) => {
            match e {
                Error::ResponseError(e) => {},
                Error::Reqwest(e) => {},
                Error::Serde(e) => {},
                Error::Io(e) => {},
            }
        }
    }
}