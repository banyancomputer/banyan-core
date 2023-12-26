use crate::database::{DatabaseConnection, BIND_LIMIT};

/// This table represents blocks that have been marked for expiration by an upload, it does not
/// represent the blocks contained within a particular metadata. This association is present to
/// delay when we mark blocks as expired (wait until the metadata becomes current).
pub struct PendingExpiration;

impl PendingExpiration {
    pub async fn record_pending_block_expirations(
        conn: &mut DatabaseConnection,
        metadata_id: &str,
        block_cids: &Vec<String>,
    ) -> Result<(), sqlx::Error> {
        let mut block_ids: Vec<String> = Vec::new();

        for cid_chunk in block_cids.chunks(BIND_LIMIT) {
            let mut block_id_builder = sqlx::QueryBuilder::new(
                r#"SELECT b.id FROM blocks AS b
                    JOIN block_locations AS bl ON bl.block_id = b.id
                    WHERE bl.metadata_id = "#,
            );
            block_id_builder.push_bind(metadata_id);
            block_id_builder.push(" AND b.cid IN (");

            let mut separated_values = block_id_builder.separated(", ");
            for cid in cid_chunk {
                separated_values.push_bind(cid);
            }

            block_id_builder.push(");");

            let queried_ids: Vec<String> = block_id_builder
                .build_query_scalar()
                .persistent(false)
                .fetch_all(&mut *conn)
                .await?;

            block_ids.extend(queried_ids);
        }

        for chunk in block_ids.chunks(BIND_LIMIT / 2) {
            let mut pending_association_query =
                sqlx::QueryBuilder::new("INSERT INTO pending_expirations (metadata_id, block_id) ");

            pending_association_query.push_values(chunk, |mut paq, bid| {
                paq.push_bind(metadata_id);
                paq.push_bind(bid);
            });

            pending_association_query.push(";");
            pending_association_query
                .build()
                .persistent(false)
                .execute(&mut *conn)
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::database::models::MetadataState;
    use crate::database::test_helpers::*;

    #[tokio::test]
    async fn test_pending_expiration_block_association() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;
        let metadata_id = sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Current).await;
        let storage_host_id = create_storage_host(&mut conn, "SP", "https://[::1]:8001/", 0).await;

        let blk_cids: Vec<_> = normalize_cids(generate_cids(data_generator(0..3))).collect();
        let mut blk_ids = create_blocks(&mut conn, blk_cids.iter().map(String::as_str)).await;

        // make comparison consistent later on
        blk_ids.sort();

        associate_blocks(
            &mut conn,
            &metadata_id,
            &storage_host_id,
            blk_ids.iter().map(String::as_str),
        )
        .await;

        PendingExpiration::record_pending_block_expirations(&mut conn, &metadata_id, &blk_cids)
            .await
            .expect("recording success");

        let mut pending_block_list = sqlx::query_scalar!(
            "SELECT block_id FROM pending_expirations WHERE metadata_id = $1;",
            metadata_id
        )
        .fetch_all(&mut *conn)
        .await
        .expect("retrieving pending expirations");

        // more consistency
        pending_block_list.sort();

        assert_eq!(pending_block_list, blk_ids);
    }
}
