-- Add up migration script here
CREATE TABLE blocks (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  cid VARCHAR(64) NOT NULL,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Enforce that the CID is unique
CREATE UNIQUE INDEX idx_blocks_on_cid
  ON blocks(cid);

CREATE TABLE block_locations (
  metadata_id TEXT NOT NULL REFERENCES metadata(id) ON DELETE CASCADE,
  block_id UUID NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
--   TODO: This should be using the storage host id
  storage_host_name TEXT NOT NULL REFERENCES storage_hosts(name) ON DELETE CASCADE,

  associated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_uploads_blocks_on_metadata_id_block_id_storage_host_name
  ON block_locations(metadata_id, block_id, storage_host_name);