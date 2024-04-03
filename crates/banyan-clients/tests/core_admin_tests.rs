use std::fmt::{Debug, Display};
use std::time::Duration;
use banyan_clients::core_admin::{Client, types};
use banyan_clients::core_admin::types::SelectedStorageHostRequest;


#[tokio::test]
async fn test_example_call() {
    let mut val = reqwest::header::HeaderValue::from_static("super-secret");
    val.set_sensitive(true);
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::AUTHORIZATION, val);
    let client_builder = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::new(60, 0))
        .default_headers(headers)
        .build()
        .unwrap();
    let client =  Client::new_with_client("https://beta.data.banyan.computer/api/v1", client_builder);
    let storage_host  = client.create_storage_host(&SelectedStorageHostRequest {
        available_storage: None,
        name: Some("name".to_string()),
        region: Some("region".to_string()),
        url: Some("url".to_string()),
    }).await;
    match storage_host {
        Ok(res) => {
            println!("{}", res.id.unwrap().to_string());
        },
        Err(e) => {
            e.to_string();
        }
    }
}