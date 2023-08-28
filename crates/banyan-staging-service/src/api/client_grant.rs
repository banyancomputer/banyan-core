use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use crate::extractors::StorageGrant;

mod service {
    use std::ops::Deref;
    use std::sync::Arc;

    use axum::extract::{FromRef, FromRequestParts};
    use http::request::Parts;

    use crate::database::Database;

    pub type DynGrantService = Arc<dyn GrantService + Send + Sync>;

    #[axum::async_trait]
    pub trait GrantService {
        async fn record_authorization(&self, public_key: String) -> Result<(), GrantServiceError>;
    }

    #[derive(Debug, thiserror::Error)]
    pub enum GrantServiceError {
        #[error("placeholder")]
        Placeholder,
    }

    struct DbGrantService {
        db: Database,
    }

    #[axum::async_trait]
    impl GrantService for DbGrantService {
        async fn record_authorization(&self, _public_key: String) -> Result<(), GrantServiceError> {
            Ok(())
        }
    }

    pub struct StateGrantService(DynGrantService);

    impl Deref for StateGrantService {
        type Target = DynGrantService;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    use crate::app::State;

    #[axum::async_trait]
    impl FromRequestParts<State> for StateGrantService {
        type Rejection = ();

        async fn from_request_parts(
            _parts: &mut Parts,
            state: &State,
        ) -> Result<Self, Self::Rejection> {
            Ok(StateGrantService(Arc::new(DbGrantService { db: Database::from_ref(state) })))
        }
    }
}

use service::{GrantService, GrantServiceError, StateGrantService};

#[derive(Deserialize, Serialize)]
pub struct GrantRequest {
    public_key: String,
}

use crate::database::Database;

#[axum::debug_handler]
pub async fn handler(
    //service: StateGrantService,
    //database: Database,
    Json(_request): Json<GrantRequest>,
) -> Response {
    //match service.record_authorization("test".to_string()).await {
    //    Ok(_) => {
    //        (StatusCode::NO_CONTENT, ()).into_response()
    //    }
    //    Err(GrantServiceError::Placeholder) => {
    //        let msg = serde_json::json!({"msg": "a placeholder error occurred"});
    //        (StatusCode::INTERNAL_SERVER_ERROR, Json(msg)).into_response()
    //    }
    //}

    let msg = serde_json::json!({"msg": "success"});
    (StatusCode::NO_CONTENT, axum::Json(msg)).into_response()
}
