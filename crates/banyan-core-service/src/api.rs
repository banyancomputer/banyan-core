use axum::{Json, Router};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use http::header::{ACCEPT, AUTHORIZATION, ORIGIN};
use http::Method;
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod buckets;

use crate::util::collect_error_messages;

pub fn router() -> Router {
    // todo: Ideally this would have a wrapper method to allow per route method configuration or
    // even better something that inspected the route matches and applied the correct method config
    // for that path...
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::DELETE, Method::POST, Method::PUT])
        .allow_headers(vec![AUTHORIZATION, ACCEPT, ORIGIN])
        // todo: add domain as a config option and make this configurable
        .allow_origin(Any)
        .allow_credentials(false);

    Router::new()
        .nest("/auth", auth::router())
        .nest("/buckets", buckets::router())
        .layer(cors_layer)
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub errors: Vec<String>,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}

macro_rules! from_error_impl {
    ($t: ty) => {
        impl From<$t> for ErrorResponse {
            fn from(err: $t) -> Self {
                Self {
                    errors: collect_error_messages(err),
                }
            }
        }
    };
}

from_error_impl!(auth::AuthError);
from_error_impl!(buckets::BucketError);
