use axum::routing::{get, post};
use axum::Router;
use http::header::{ACCEPT, ORIGIN};
use http::{HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};

mod block_retrieval;
mod client_grant;
mod upload;

use crate::app::State;

pub fn router(state: State) -> Router<State> {
    //let cors_layer = CorsLayer::new()
    //    .allow_methods(vec![Method::GET, Method::POST])
    //    .allow_headers(vec![ACCEPT, ORIGIN])
    //    .allow_origin(vec![state.platform_base_url().as_str().parse().unwrap()])
    //    .allow_credentials(true);

    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .route("/cors_test", get(cors_test))
        .route("/blocks/:block_id", get(block_retrieval::handler))
        .route("/client_grant", post(client_grant::handler))
        .route("/upload", post(upload::handler))
        .layer(cors_layer)
        .with_state(state)
}

use axum::response::IntoResponse;

pub async fn cors_test() -> axum::response::Response {
    let msg = serde_json::json!({"status": "ok"});
    (http::StatusCode::OK, axum::Json(msg)).into_response()
}
