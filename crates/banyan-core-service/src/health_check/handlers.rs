use axum::extract::State;
use banyan_task::{SqliteTaskStore, TaskStore};

use crate::app::{AppState, Version};
use crate::health_check::{HealthCheckError, HealthCheckResponse, HealthCheckService};

pub async fn liveness_check() -> HealthCheckResponse {
    HealthCheckResponse::Ready
}

pub async fn readiness_check(mut healthcheck_service: HealthCheckService) -> HealthCheckResponse {
    if let Err(err) = healthcheck_service.ready().await {
        return err.into();
    }

    HealthCheckResponse::Ready
}

pub async fn task_store_metrics(State(state): State<AppState>) -> HealthCheckResponse {
    let task_store = SqliteTaskStore::new(state.database());
    match task_store.metrics().await {
        Ok(metrics) => HealthCheckResponse::TaskStoreMetrics(metrics),
        Err(err) => HealthCheckError::TaskStoreError(err).into(),
    }
}

pub async fn version() -> Version {
    Version::new()
}
