use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::response::Response;
use jwt_simple::algorithms::ES384KeyPair;

use crate::app::{AppState, ProviderCredential, Secrets, ServiceKey, ServiceVerificationKey};
use crate::database::Database;
use crate::event_bus::EventBus;

pub fn mock_app_state(database: Database) -> State<AppState> {
    let mut provider_creds = std::collections::BTreeMap::new();
    provider_creds.insert(
        Arc::from("mock_provider"),
        ProviderCredential::new("mock_pem", "secret"),
    );
    State(AppState {
        database,
        event_bus: EventBus::default(),
        secrets: Secrets::new(
            provider_creds,
            None,
            ServiceKey::new(ES384KeyPair::generate()),
        ),
        service_name: "mock_service".to_string(),
        service_verifier: ServiceVerificationKey::new(ES384KeyPair::generate().public_key()),
        upload_directory: PathBuf::from("/mock/path"),
    })
}

pub async fn deserialize_response<T: for<'de> serde::Deserialize<'de>, E: std::fmt::Debug>(
    res: Result<Response, E>,
) -> T {
    let res = res.unwrap();
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    return serde_json::from_slice(&body).unwrap();
}
