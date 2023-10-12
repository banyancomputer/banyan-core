use std::fmt::Display;

use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[derive(Debug)]
#[non_exhaustive]
pub struct CoreError {
    kind: CoreErrorKind,
}

impl Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.kind))
    }
}

impl std::error::Error for CoreError {}

impl CoreError {
    /// Deault error
    fn default_response(message: Option<String>) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            message.unwrap_or("internal server error".to_string()),
        )
            .into_response()
    }

    pub fn default_error(message: &str) -> Self {
        Self {
            kind: CoreErrorKind::Default(message.to_string()),
        }
    }

    pub fn sqlx_error(err: sqlx::Error, operation: &str, resource: &str) -> Self {
        Self {
            kind: CoreErrorKind::Sqlx {
                err,
                operation: operation.to_string(),
                resource: resource.to_string(),
            },
        }
    }

    pub fn generic_error(code: StatusCode, msg: &str, trace: Option<&str>) -> Self {
        Self {
            kind: CoreErrorKind::Generic {
                code,
                msg: msg.to_string(),
                trace: trace.map(|trace| trace.to_string()),
            },
        }
    }
}

#[derive(Debug)]
pub enum CoreErrorKind {
    /// Generic 500 Error with message
    Default(String),
    /// SQLX Error
    Sqlx {
        /// Error
        err: sqlx::Error,
        /// Operation
        operation: String,
        /// Resource
        resource: String,
    },
    Generic {
        /// Status Code
        code: StatusCode,
        /// Message
        msg: String,
        /// Trace
        trace: Option<String>,
    },
}

impl IntoResponse for CoreError {
    fn into_response(self) -> axum::response::Response {
        match self.kind {
            CoreErrorKind::Default(message) => {
                tracing::error!("{message}");
                Self::default_response(Some(message))
            }
            // Sqlx Error
            CoreErrorKind::Sqlx {
                err,
                operation,
                resource,
            } => {
                tracing::error!("unable to {operation} {resource} {{ {err} }}");
                match err {
                    sqlx::Error::Database(db_err) => {
                        // If this is duplicate
                        if db_err.is_unique_violation() {
                            (
                                StatusCode::CONFLICT,
                                format!("{} with that name already exists", resource),
                            )
                                .into_response()
                        } else {
                            Self::default_response(None)
                        }
                    }
                    sqlx::Error::RowNotFound => (
                        StatusCode::NOT_FOUND,
                        format!("{} not found: {}", resource, err),
                    )
                        .into_response(),
                    // Catch all others
                    _ => Self::default_response(None),
                }
            }
            CoreErrorKind::Generic { code, msg, trace } => {
                if let Some(trace) = trace {
                    tracing::error!("{trace}")
                }
                (code, axum::extract::Json(serde_json::json!({"msg": msg}))).into_response()
            }
        }
    }
}
