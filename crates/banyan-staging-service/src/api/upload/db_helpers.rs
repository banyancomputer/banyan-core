use banyan_car_analyzer::CarReport;
use sqlx::prelude::FromRow;
use sqlx::Decode;
use uuid::Uuid;

use super::error::UploadError;
use crate::database::{map_sqlx_error, Database};

pub async fn start_upload(
    db: &Database,
    client_id: &Uuid,
    metadata_id: &Uuid,
    reported_size: u64,
) -> Result<Upload, UploadError> {
    let blocks_path = std::path::Path::new(&format!("{metadata_id}"))
        .display()
        .to_string();

    let upload = sqlx::query_as(
        r#"
        INSERT INTO
            uploads (client_id, metadata_id, reported_size, blocks_path, state)
            VALUES ($1, $2, $3, $4, 'started');
        "#,
    )
    .bind(client_id.to_string())
    .bind(metadata_id.to_string())
    .bind(reported_size as i64)
    .bind(&blocks_path)
    .fetch_one(db)
    .await
    .map_err(map_sqlx_error)
    .map_err(UploadError::Database)?;
    Ok(upload)
}

/// Marks an upload as failed
pub async fn fail_upload(db: &Database, upload_id: &str) -> Result<(), UploadError> {
    let _rows_affected: i32 = sqlx::query_scalar(
        r#"
        UPDATE uploads SET state = 'failed' WHERE id = $1;
        "#,
    )
    .bind(upload_id)
    .fetch_one(db)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

pub async fn complete_upload(
    db: &Database,
    total_size: i64,
    integrity_hash: &str,
    upload_id: &str,
) -> Result<(), UploadError> {
    //let mut path_iter = file_path.parts();
    // discard the uploading/ prefix
    //let _ = path_iter.next();
    //let mut final_path: object_store::path::Path = path_iter.collect();

    //// todo: validate the local store doesn't use copy to handle this as some of the other stores
    //// do...
    //if let Err(err) = store.rename_if_not_exists(file_path, &final_path).await {
    //    // this is a weird error, its successfully written to our file store so we have it and can
    //    // serve it we just want to make sure we don't update the path in the DB and clean it up
    //    // later
    //    tracing::error!("unable to rename upload, leaving it in place: {err}");
    //    // todo: background a task to handle correcting this issue when it occurs
    //    final_path = file_path.clone();
    //}

    // todo: should definitely do a db transaction before the attempted rename, committing only if
    // the rename is successfuly
    sqlx::query(
        r#"
                    UPDATE uploads SET
                            state = 'complete',
                            final_size = $1,
                            integrity_hash = $2
                        WHERE id = $3;
                "#,
    )
    .bind(total_size)
    .bind(integrity_hash)
    .bind(upload_id)
    .execute(db)
    .await
    .map_err(map_sqlx_error)?;

    Ok(())
}

pub async fn get_upload(
    db: &Database,
    client_id: Uuid,
    metadata_id: Uuid,
) -> Result<Option<Upload>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT * FROM uploads
            WHERE client_id = $1
            AND metadata_id = $2;
        "#,
    )
    .bind(client_id.to_string())
    .bind(metadata_id.to_string())
    .fetch_optional(db)
    .await
}

#[derive(sqlx::FromRow, sqlx::Decode)]
pub struct Upload {
    pub id: String,
    pub client_id: String,
    pub metadata_id: String,
    pub reported_size: i64,
    pub blocks_path: String,
    pub state: String,
}
