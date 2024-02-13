use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

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

    pub fn serde(&self) -> SerdeVersion {
        SerdeVersion {
            build_profile: String::from(self.build_profile),
            build_timestamp: String::new(),
            features: self.features.iter().map(|s| s.to_string()).collect(),
            version: String::from(self.version),
        }
    }
}

impl IntoResponse for Version {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self.serde())).into_response()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerdeVersion {
    pub build_profile: String,
    pub build_timestamp: String,
    pub features: Vec<String>,
    pub version: String,
}
