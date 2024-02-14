use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Version {
    pub build_profile: &'static str,
    pub features: Vec<&'static str>,
    pub version: &'static str,
}

impl Version {
    pub fn new() -> Self {
        Self {
            build_profile: env!("BUILD_PROFILE"),
            features: env!("BUILD_FEATURES").split(',').collect::<Vec<_>>(),
            version: env!("REPO_VERSION"),
        }
    }
}

impl IntoResponse for Version {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
