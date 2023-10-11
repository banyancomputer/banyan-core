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

use axum::Json;
use axum::extract::{FromRef, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use sqlx::SqlitePool;

use crate::app_state::AppState;
use crate::workers::{SqliteTaskStore, TaskLikeExt};
use crate::workers::tasks::TestTask;

pub async fn work_test(State(state): State<AppState>) -> Response {
    let mut pool = SqlitePool::from_ref(&state);

    TestTask::new(uuid::Uuid::new_v4())
        .enqueue::<SqliteTaskStore>(&mut pool)
        .await
        .expect("queue success");

    (StatusCode::OK, Json(serde_json::json!({"msg": "ok"}))).into_response()
}
