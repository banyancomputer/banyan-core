use crate::health_check::{HealthCheckResponse, HealthCheckService, VersionResponse};

pub async fn liveness_check() -> HealthCheckResponse {
    HealthCheckResponse::Ready
}

pub async fn readiness_check(mut healthcheck_service: HealthCheckService) -> HealthCheckResponse {
    if let Err(err) = healthcheck_service.ready().await {
        return err.into();
    }

    HealthCheckResponse::Ready
}

pub async fn version() -> VersionResponse<'static> {
    VersionResponse {
        build_profile: env!("BUILD_PROFILE"),
        features: env!("BUILD_FEATURES").split(',').collect(),
        version: env!("REPO_VERSION"),
    }
}
