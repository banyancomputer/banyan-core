use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::api::models::ApiUserKey;
use crate::app::AppState;
use crate::database::models::{Bucket, UserKey};
use crate::extractors::UserIdentity;

use axum::Json as AxumJson;
use sqlx::types::Json as SqlxJson;

#[derive(Deserialize)]
pub struct Key {
    id: String,
    user_id: String,
}

type AssociatedKeys = SqlxJson<Vec<UserKey>>;

#[derive(sqlx::FromRow)]
pub struct Thing {
    pub id: String,
    pub user_id: String,
    pub pem: String,
    pub fingerprint: String,
    pub bucket_ids: SqlxJson<Vec<String>>,
}

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
) -> Result<Response, UserKeyAccessError> {
    let database = state.database();

    let user_id = user_identity.id().to_string();

    // get all bucket ids for buckets which we can access
    let relevant_bucket_ids: Vec<String> = sqlx::query_scalar!(
        r#"
            SELECT b.id FROM buckets AS b
            JOIN bucket_access AS ba ON ba.bucket_id = b.id
            JOIN user_keys AS uk ON uk.id = ba.user_key_id
            WHERE uk.user_id = $1
        "#,
        user_id
    )
    .fetch_all(&database)
    .await?;

    let things = sqlx::query_as!(
        Thing,
        r#"
            SELECT 
                uk.id, 
                uk.user_id, 
                uk.pem, 
                uk.fingerprint, 
                FORMAT(
                    '[%s]',
                    GROUP_CONCAT(
                        FORMAT('"%s"', b.id)
                    )
                ) AS "bucket_ids!: SqlxJson<Vec<String>>"
            FROM user_keys AS uk
            JOIN bucket_access AS ba ON ba.user_key_id = uk.id
            JOIN buckets AS b ON b.id = ba.bucket_id WHERE b.id IN (
                SELECT b2.id FROM buckets AS b2
                JOIN bucket_access AS ba2 ON ba2.bucket_id = b2.id
                JOIN user_keys AS uk2 ON uk2.id = ba2.user_key_id
                WHERE uk2.user_id=$1
            ) GROUP BY uk.id
            UNION
            SELECT 
                uk.id, 
                uk.user_id,
                uk.fingerprint,
                uk.pem, '[]' as "bucket_ids!: SqlxJson<Vec<String>>"
            FROM user_keys AS uk 
            LEFT OUTER JOIN bucket_access AS ba ON ba.user_key_id = uk.id
            WHERE ba.user_key_id IS NULL
            AND uk.user_id=$1;
        "#,
        user_id,
    )
    .fetch_all(&database)
    .await?;

    tracing::info!("rvbids: {:?}", relevant_bucket_ids);

    let mut conn = database.acquire().await?;
    for bucket_id in relevant_bucket_ids {
        Bucket::list_user_keys(&mut conn, &bucket_id).await?;
    }

    /*
    // query to get all user keys that a user should be able to view
    //
    //
    r#"
        select from uk
        join on ba

        bucket access has a keyid owned by us
        & bucket.user_id != us


        /// select all bucket access entries where the associated key is owned by us
        /// but the bucket it's associated with is NOT owned by us
        ///
        ///
        /// select all user keys owned by us
        let ourkeys =
        SELECT *
        FROM user_keys
        WHERE user_id = $1;


        /// get all keys which correspond to a bucket we care about
        SELECT * FROM user_keys AS uk
        JOIN bucket_access AS ba ON ba.user_key_id = uk.id
        JOIN buckets AS b ON b.id = ba.bucket_id
        WHERE b.id IN bbucks
        "#

    let query_result = sqlx::query_as!(
        UserKey,
        r#"

            SELECT uk.*
            FROM bucket_access AS ba
            JOIN buckets AS b ON b.id = ba.bucket_id
            JOIN user_keys AS uk ON uk.id = ba.user_key_id
            WHERE b.id IN (




            )




            SELECT *
            FROM bucket_access AS ba
            JOIN buckets AS b ON b.id = ba.bucket_id
            JOIN user_keys AS uk ON uk.id = ba.user_key_id
            // where its one they own
            WHERE uk.user_id = $1
            // or its one that's been shared with them
            OR ba.user_key_id.userkey.id = $1


            GROUP BY uk.id;

            //JOIN users AS u ON u.id = uk.user_id
            JOIN buckets AS b ON b.user_id = uk.user_id
            JOIN bucket_access AS ba ON ba.user_key_id = uk.id
            WHERE user_id = $1
            GROUP BY uk.id
            ;
        "#,
        user_id,
    )
    .fetch_all(&database)
    .await;
    */

    panic!("");
}

#[derive(Debug, thiserror::Error)]
pub enum UserKeyAccessError {
    #[error("database query failures: {0}")]
    DatabaseFailure(#[from] sqlx::Error),
}

impl IntoResponse for UserKeyAccessError {
    fn into_response(self) -> Response {
        match &self {
            _ => {
                tracing::error!("a stripe webhook error occurred: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, AxumJson(err_msg)).into_response()
            }
        }
    }
}
