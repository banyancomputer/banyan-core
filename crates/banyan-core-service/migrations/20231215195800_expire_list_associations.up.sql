CREATE TABLE pending_expirations (
  metadata_id TEXT NOT NULL REFERENCES metadata(id) ON DELETE CASCADE,
  block_id TEXT NOT NULL REFERENCES blocks(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_pending_expiration_on_metadata_id_block_id
  ON pending_expirations(metadata_id, block_id);
