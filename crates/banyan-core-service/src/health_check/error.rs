use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
#[non_exhaustive]
pub struct HealthCheckError;

impl HealthCheckError {
    pub fn is_temporary(&self) -> bool {
        false
    }
}

impl Display for HealthCheckError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str("an unknown healthcheck error occurred")
    }
}

impl std::error::Error for HealthCheckError {}
