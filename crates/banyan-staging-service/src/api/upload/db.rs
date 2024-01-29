use std::str::FromStr;

use banyan_task::TaskLikeExt;
use cid::Cid;
use uuid::Uuid;

use crate::database::Database;
use crate::tasks::ReportUploadTask;

pub async fn start_upload(
    db: &Database,
    client_id: &Uuid,
    metadata_id: &Uuid,
    reported_size: u64,
) -> Result<Upload, sqlx::Error> {
    let mut upload = Upload {
        id: String::new(),
        client_id: client_id.to_string(),
        metadata_id: metadata_id.to_string(),
        base_path: metadata_id.to_string(),
        reported_size: reported_size as i64,
        state: String::from("started"),
    };

    tracing::info!("base_path: {}", upload.base_path);

    upload.id = sqlx::query_scalar(
        r#"
        INSERT INTO
            uploads (client_id, metadata_id, reported_size, base_path, state)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id;
        "#,
    )
    .bind(&upload.client_id)
    .bind(&upload.metadata_id)
    .bind(upload.reported_size)
    .bind(&upload.base_path)
    .bind(&upload.state)
    .fetch_one(db)
    .await
    .map_err(|err| {
        tracing::error!("errrrrr: {err}");
    })
    .unwrap();

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
    //let mut path_iter = base_path.parts();
    // discard the uploading/ prefix
    //let _ = path_iter.next();
    //let mut final_path: object_store::path::Path = path_iter.collect();

    //// todo: validate the local store doesn't use copy to handle this as some of the other stores
    //// do...
    //if let Err(err) = store.rename_if_not_exists(base_path, &final_path).await {
    //    // this is a weird error, its successfully written to our file store so we have it and can
    //    // serve it we just want to make sure we don't update the path in the DB and clean it up
    //    // later
    //    tracing::error!("unable to rename upload, leaving it in place: {err}");
    //    // todo: background a task to handle correcting this issue when it occurs
    //    final_path = base_path.clone();
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

pub async fn report_upload(
    db: &mut Database,
    storage_grant_id: Uuid,
    metadata_id: &str,
    upload_id: &str,
) -> Result<(), sqlx::Error> {
    let all_cids: Vec<String> = sqlx::query_scalar(
        r#"
            SELECT blocks.cid 
            FROM blocks
            JOIN uploads_blocks ON blocks.id = uploads_blocks.block_id
            JOIN uploads ON uploads_blocks.upload_id = uploads.id
            WHERE uploads.id = $1;
            "#,
    )
    .bind(upload_id)
    .fetch_all(&*db)
    .await?;

    let all_cids = all_cids
        .into_iter()
        .map(|cid_string| Cid::from_str(&cid_string).unwrap())
        .collect::<Vec<Cid>>();

    let total_size: i64 = sqlx::query_scalar(
        r#"
            SELECT COALESCE(SUM(blocks.data_length), 0)
            FROM blocks
            JOIN uploads_blocks ON blocks.id = uploads_blocks.block_id
            JOIN uploads ON uploads_blocks.upload_id = uploads.id
            WHERE uploads.id = $1;
            "#,
    )
    .bind(upload_id)
    .fetch_one(&*db)
    .await?;

    ReportUploadTask::new(storage_grant_id, metadata_id, &all_cids, total_size as u64)
        .enqueue::<banyan_task::SqliteTaskStore>(db)
        .await
        .unwrap();

    Ok(())
}

pub async fn get_upload(
    db: &Database,
    client_id: Uuid,
    upload_id: &str,
) -> Result<Option<Upload>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT id, client_id, metadata_id, reported_size, state FROM uploads
            WHERE client_id = $1
            AND id = $2;
        "#,
    )
    .bind(client_id.to_string())
    .bind(upload_id)
    .fetch_optional(db)
    .await
}

pub async fn write_block_to_tables(
    db: &Database,
    upload_id: &str,
    normalized_cid: &str,
    data_length: i64,
) -> Result<(), sqlx::Error> {
    // Insert the block if its missing, get its ID
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

    // Create uploads_blocks row with the block information
    // We omit car_offset because that's only for deprecated infra
    sqlx::query(
        r#"
        INSERT INTO
            uploads_blocks (upload_id, block_id)
            VALUES ($1, $2);
        "#,
    )
    .bind(upload_id)
    .bind(block_id)
    .execute(db)
    .await
    .map(|_| ())
}

#[derive(sqlx::FromRow, sqlx::Decode)]
pub struct Upload {
    pub id: String,
    pub client_id: String,
    pub metadata_id: String,
    pub base_path: String,
    pub reported_size: i64,
    pub state: String,
}
