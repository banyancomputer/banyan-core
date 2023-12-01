use crate::database::models::{BucketType, StorageClass};

#[derive(sqlx::FromRow)]
pub struct Bucket {
    pub id: String,

    pub user_id: String,

    pub name: String,
    pub r#type: BucketType,
    pub storage_class: StorageClass,
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::database::Database;

    use super::*;

    pub(crate) async fn create_bucket(database: &Database, user_id: &str, name: &str) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!(
            r#"INSERT INTO
                   buckets (user_id, name, type, storage_class)
                   VALUES ($1, $2, $3, $4)
                   RETURNING id;"#,
            user_id,
            name,
            BucketType::Interactive,
            StorageClass::Hot,
        )
        .fetch_one(database)
        .await
    }
}
