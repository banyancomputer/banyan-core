use crate::database::DatabaseConnection;

/// This table represents blocks that have been marked for expiration by an upload, it does not
/// represent the blocks contained within a particular metadata. This association is present to
/// delay when we mark blocks as expired (wait until the metadata becomes current).
pub struct PendingExpiration;

impl PendingExpiration {
    pub async fn record_pending_block_expirations(
        conn: &mut DatabaseConnection,
        metadata_id: &str,
        block_cids: impl IntoIterator<Item = &str>,
    ) -> Result<(), sqlx::Error> {
        let mut block_cid_iterator = block_cids.into_iter().peekable();
        if block_cid_iterator.peek().is_none() {
            return Ok(());
        }

        let mut block_id_builder = sqlx::QueryBuilder::new(
            r#"SELECT b.id FROM blocks AS b
                  JOIN block_locations AS bl ON bl.block_id = b.id
                  WHERE bl.metadata_id = $1
                      AND b.cid IN ("#,
        );

        while let Some(cid) = block_cid_iterator.next() {
            block_id_builder.push_bind(cid);

            if block_cid_iterator.peek().is_some() {
                block_id_builder.push(", ");
            }
        }

        block_id_builder.push(");");
        let block_ids: Vec<String> = block_id_builder
            .build_query_scalar()
            .persistent(false)
            .fetch_all(&mut *conn)
            .await?;

        // We could end up with no blocks if the provided ones couldn't be found
        if block_ids.is_empty() {
            tracing::warn!("all blocks filtered out due to being unknown");
            return Ok(());
        }

        let mut pending_association_query = sqlx::QueryBuilder::new(
            "INSERT INTO pending_expirations (metadata_id, block_id) VALUES ",
        );

        pending_association_query.push_tuples(block_ids, |mut paq, bid| {
            paq.push_bind(metadata_id);
            paq.push_bind(bid);
        });

        pending_association_query.push(";");
        tracing::info!("pending association query sql: {}", pending_association_query.sql());
        pending_association_query
            .build()
            .execute(&mut *conn)
            .await?;

        Ok(())
    }
}
