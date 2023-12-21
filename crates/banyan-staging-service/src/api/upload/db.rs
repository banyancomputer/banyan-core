use uuid::Uuid;

use crate::database::Database;

pub async fn start_upload(
    db: &Database,
    client_id: &Uuid,
    metadata_id: &Uuid,
    reported_size: u64,
) -> Result<Upload, sqlx::Error> {
    let blocks_path = std::path::Path::new(&format!("{metadata_id}"))
        .display()
        .to_string();

    let mut upload = Upload {
        id: String::new(),
        client_id: client_id.to_string(),
        metadata_id: metadata_id.to_string(),
        reported_size: reported_size as i64,
        blocks_path,
        state: String::from("started"),
    };

    upload.id = sqlx::query_scalar!(
        r#"
        INSERT INTO
            uploads (client_id, metadata_id, reported_size, blocks_path, state)
            VALUES ($1, $2, $3, $4, 'started')
            RETURNING id;
        "#,
        upload.client_id,
        upload.metadata_id,
        upload.reported_size,
        upload.blocks_path
    )
    .fetch_one(db)
    .await?;

    Ok(upload)
}

/// Marks an upload as failed
pub async fn fail_upload(db: &Database, upload_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE uploads SET state = 'failed' WHERE id = $1;
        "#,
    )
    .bind(upload_id)
    .execute(db)
    .await
    .map(|_| ())
}

pub async fn complete_upload(
    db: &Database,
    total_size: i64,
    integrity_hash: &str,
    upload_id: &str,
) -> Result<(), sqlx::Error> {
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
    .map(|_| ())
}

pub async fn get_upload(
    db: &Database,
    client_id: Uuid,
    metadata_id: Uuid,
) -> Result<Option<Upload>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT id, client_id, metadata_id, reported_size, blocks_path, state FROM uploads
            WHERE client_id = $1
            AND metadata_id = $2;
        "#,
    )
    .bind(client_id.to_string())
    .bind(metadata_id.to_string())
    .fetch_optional(db)
    .await
}

pub async fn write_block_to_tables(
    db: &Database,
    upload_id: &str,
    normalized_cid: &str,
    data_length: i64,
    offset: i64,
) -> Result<(), sqlx::Error> {
    let block_id: String = sqlx::query_scalar(
        r#"
        INSERT OR IGNORE INTO
            blocks (cid, data_length)
            VALUES ($1, $2)
        RETURNING id;
        "#,
    )
    .bind(normalized_cid)
    .bind(data_length)
    .fetch_one(db)
    .await?;

    // let block_id: Uuid = {
    //     let cid_id: String =
    //         sqlx::query_scalar("SELECT id FROM blocks WHERE cid = $1 LIMIT 1;")
    //             .bind(cid_string.clone())
    //             .fetch_one(db)
    //             .await?;

    //     Uuid::parse_str(&cid_id)
    //         .map_err(|_| UploadStreamError::DatabaseCorruption("cid uuid parsing"))?
    // };

    // // create uploads_blocks row with the block information
    sqlx::query(
        r#"
        INSERT INTO
            uploads_blocks (upload_id, block_id, byte_offset)
            VALUES ($1, $2, $3);
        "#,
    )
    .bind(upload_id)
    .bind(block_id)
    .bind(offset)
    .execute(db)
    .await?;

    Ok(())
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
