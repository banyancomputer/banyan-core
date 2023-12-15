/// The triple of these attributes make up the unique association ID for the `block_locations`
/// table. This structure is appropriate to use whenever one or more of these rows needs to be
/// uniquely identified without the associated metadata on the link.
#[derive(sqlx::FromRow)]
pub struct MinimalBlockLocation {
    pub block_id: String,
    pub metadata_id: String,
    pub storage_host_id: String,
}
