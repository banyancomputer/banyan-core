use crate::health_check::{HealthCheckResponse, HealthCheckService};

pub async fn liveness_check() -> HealthCheckResponse {
    HealthCheckResponse::Ready
}

pub async fn readiness_check(mut healthcheck_service: HealthCheckService) -> HealthCheckResponse {
    if let Err(err) = healthcheck_service.ready().await {
        return err.into();
    }

    HealthCheckResponse::Ready
}
