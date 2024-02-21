use std::str::FromStr;
use std::time::Duration;

use banyan_task::TaskLikeExt;
use cid::Cid;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::database::Database;
use crate::tasks::ReportUploadTask;

pub const UPLOAD_SESSION_DURATION: Duration = Duration::from_secs(60 * 60 * 6);

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

    upload.id = sqlx::query_scalar!(
        r#"
        INSERT INTO
            uploads (client_id, metadata_id, reported_size, base_path, state, created_at)
            VALUES ($1, $2, $3, $4, $5, DATETIME('now'))
            RETURNING id;
        "#,
        upload.client_id,
        upload.metadata_id,
        upload.reported_size,
        upload.base_path,
        upload.state,
    )
    .fetch_one(db)
    .await?;

    Ok(upload)
}

/// Marks an upload as failed
pub async fn fail_upload(db: &Database, upload_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE uploads SET state = 'failed' WHERE id = $1;
        "#,
        upload_id
    )
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
    sqlx::query!(
        r#"
        UPDATE uploads SET
                state = 'complete',
                final_size = $1,
                integrity_hash = $2,
                finished_at = DATETIME('now')
            WHERE id = $3;
        "#,
        total_size,
        integrity_hash,
        upload_id,
    )
    .execute(db)
    .await
    .map(|_| ())
}

pub async fn upload_size(db: &Database, upload_id: &str) -> Result<i64, sqlx::Error> {
    let total_size: i32 = sqlx::query_scalar!(
        r#"
            SELECT COALESCE(SUM(blocks.data_length), 0)
            FROM blocks
            JOIN uploads_blocks ON blocks.id = uploads_blocks.block_id
            JOIN uploads ON uploads_blocks.upload_id = uploads.id
            WHERE uploads.id = $1;
            "#,
        upload_id
    )
    .fetch_one(db)
    .await?;

    Ok(total_size as i64)
}

pub async fn report_upload(
    db: &mut Database,
    storage_grant_id: Uuid,
    metadata_id: &str,
    upload_id: &str,
    total_size: i64,
) -> Result<(), sqlx::Error> {
    let all_cids: Vec<String> = sqlx::query_scalar!(
        r#"
            SELECT blocks.cid 
            FROM blocks
            JOIN uploads_blocks ON blocks.id = uploads_blocks.block_id
            JOIN uploads ON uploads_blocks.upload_id = uploads.id
            WHERE uploads.id = $1;
        "#,
        upload_id
    )
    .fetch_all(&*db)
    .await?;

    let all_cids = all_cids
        .into_iter()
        .map(|cid_string| Cid::from_str(&cid_string).unwrap())
        .collect::<Vec<Cid>>();

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
    let client_id = client_id.to_string();
    let now = OffsetDateTime::now_utc() - UPLOAD_SESSION_DURATION;

    sqlx::query_as!(
        Upload,
        r#"
        SELECT id, client_id, metadata_id, base_path, reported_size, state FROM uploads
            WHERE client_id = $1
            AND id = $2
            AND created_at >= $3;
        "#,
        client_id,
        upload_id,
        now
    )
    .fetch_optional(db)
    .await
}

pub async fn write_block_to_tables(
    db: &Database,
    upload_id: &str,
    normalized_cid: &str,
    data_length: i64,
) -> Result<(), sqlx::Error> {
    let maybe_block_id: Option<String> = sqlx::query_scalar!(
        "INSERT OR IGNORE INTO blocks (cid, data_length) VALUES ($1, $2) RETURNING id;",
        normalized_cid,
        data_length,
    )
    .fetch_optional(db)
    .await?;

    let block_id = match maybe_block_id {
        Some(block_id) => block_id,
        None => {
            sqlx::query_scalar!("SELECT id FROM blocks WHERE cid = $1;", normalized_cid,)
                .fetch_one(db)
                .await?
        }
    };

    // Create uploads_blocks row with the block information
    // We omit car_offset because that's only for deprecated infra
    sqlx::query!(
        r#"
        INSERT INTO
            uploads_blocks (upload_id, block_id)
            VALUES ($1, $2);
        "#,
        upload_id,
        block_id,
    )
    .execute(db)
    .await?;

    Ok(())
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
