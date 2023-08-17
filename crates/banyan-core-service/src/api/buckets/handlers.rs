use axum::extract::{self, Json, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;
use validify::Validate;

use crate::db::*;
use crate::api::buckets::{requests, responses, keys};
use crate::extractors::{ApiToken, DbConn};

/// Initialze a new bucket with initial key material
pub async fn create(
    api_token: ApiToken,
    mut db_conn: DbConn,
    extract::Json(new_bucket): extract::Json<requests::CreateBucket>,
) -> impl IntoResponse {
    if let Err(errors) = new_bucket.validate() {
        return (
            StatusCode::BAD_REQUEST,
            format!("errors: {:?}", errors.errors()),
        )
            .into_response();
    }

    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO buckets (account_id, name, type) VALUES ($1, $2, $3) RETURNING id;"#,
        api_token.subject,
        new_bucket.name,
        new_bucket.r#type,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let created_bucket = match maybe_bucket {
        Ok(cb) => cb,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to create new bucket: {err}"),
            )
                .into_response();
        }
    };

    let maybe_key = sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO bucket_keys (bucket_id, approved, pem) VALUES ($1, true, $2) RETURNING id;"#,
        created_bucket.id,
        new_bucket.initial_bucket_key_pem,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let created_key = match maybe_key {
        Ok(ck) => ck,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to create public key associated with bucket: {err}"),
            )
                .into_response();
        }
    };

    let response = responses::CreateBucket {
        id: created_bucket.id,
        name: new_bucket.name,
        r#type: new_bucket.r#type,
        initial_bucket_key: keys::responses::CreateBucketKey {
            id: created_key.id,
            approved: true,
        },
    };

    (StatusCode::OK, axum::Json(response)).into_response()
}

// TODO: pagination
/// Read all buckets associated with the calling account
pub async fn read_all(
    api_token: ApiToken,
    mut db_conn: DbConn,
) -> impl IntoResponse { 
    let account_id = api_token.subject;
    let maybe_buckets = sqlx::query_as!(
        models::Bucket,
        r#"SELECT id as "id!", account_id, name, type FROM buckets WHERE account_id = $1"#,
        account_id,
    )
    .fetch_all(&mut *db_conn.0)
    .await;

    let buckets = match maybe_buckets {
        Ok(buckets) => buckets,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read buckets: {err}"),
            )
                .into_response();
        }
    };

    let buckets = buckets.into_iter().map(|bucket| {
        responses::ReadBucket {
            id: bucket.id,
            name: bucket.name,
            r#type: bucket.r#type,
        }
    }).collect::<Vec<_>>();
    let buckets = responses::ReadBuckets(buckets);
    Json(buckets).into_response()
}

// TODO: Should this be authenticated or not?
/// Read a single bucket by id. Also search and return by account id
pub async fn read(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let maybe_bucket = sqlx::query_as!(
        models::Bucket,
        r#"SELECT id as "id!", account_id, name, type FROM buckets WHERE id = $1 AND account_id = $2"#,
        bucket_id,
        account_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let bucket = match maybe_bucket {
        Ok(bucket) => responses::ReadBucket {
            id: bucket.id,
            name: bucket.name,
            r#type: bucket.r#type,
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket: {err}"),
            )
                .into_response();
        }
    };

    Json(bucket).into_response()
}


pub async fn delete(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let maybe_bucket = sqlx::query_as!(
        models::Bucket,
        r#"DELETE FROM buckets WHERE id = $1 AND account_id = $2 RETURNING id as "id!", account_id, name, type"#,
        bucket_id,
        account_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    let bucket = match maybe_bucket {
        Ok(bucket) => responses::DeleteBucket {
            id: bucket.id,
            name: bucket.name,
            r#type: bucket.r#type,
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to delete bucket: {err}"),
            )
                .into_response();
        }
    };

    Json(bucket).into_response()
}

