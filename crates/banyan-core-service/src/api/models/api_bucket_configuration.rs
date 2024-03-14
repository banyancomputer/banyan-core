use serde::Deserialize;
use validify::{schema_err, schema_validation, ValidationErrors, Validify};

#[derive(Clone, Debug, Deserialize, Validify)]
#[validate(validate_at_least_one_field)]
pub struct ApiBucketConfiguration {
    #[validate(length(min = 3, max = 32))]
    pub name: Option<String>,
    #[validate(range(min = 1., max = 5.))]
    pub replicas: Option<i64>,
}

#[schema_validation]
fn validate_at_least_one_field(config: &ApiBucketConfiguration) -> Result<(), ValidationErrors> {
    if config.name.is_none() && config.replicas.is_none() {
        schema_err!(
            "Invalid bucket configuration",
            "at least one field should be set"
        );
    }
}
