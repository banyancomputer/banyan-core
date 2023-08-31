CREATE TABLE clients (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  platform_id TEXT NOT NULL,
  fingerprint VARCHAR(64) NOT NULL,
  public_key TEXT NOT NULL,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_clients_on_fingerprint
  ON clients(fingerprint);

CREATE TABLE storage_grants (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  client_id TEXT NOT NULL REFERENCES clients(id),
  remote_grant_id TEXT NOT NULL,
  allowed_storage INT NOT NULL DEFAULT 0,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_storage_grants_on_remote_grant_id
  ON storage_grants(remote_grant_id);
CREATE INDEX idx_storage_grants_on_created_at
  ON storage_grants(created_at);

CREATE TABLE uploads (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  client_id TEXT NOT NULL REFERENCES clients(id),
  metadata_id TEXT NOT NULL,

  reported_size INTEGER NOT NULL CHECK (reported_size >= 0) CONSTRAINT reported_size_positive,
  final_size INTEGER NOT NULL DEFAULT 0 CHECK (reported_size >= 0) CONSTRAINT final_size_positive,

  file_path VARCHAR(128) NOT NULL,
  state VARCHAR(32) NOT NULL CHECK (state IN ('started', 'indexing', 'complete', 'failed')) CONSTRAINT state_in_list,
  integrity_hash VARCHAR(32),

  started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  finished_at TIMESTAMP
);

CREATE INDEX idx_uploads_on_client_id
  ON uploads(client_id);
CREATE UNIQUE INDEX idx_uploads_on_metadata_id
  ON uploads(metadata_id);

CREATE TABLE blocks (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  cid VARCHAR(64) NOT NULL,
  owner_id TEXT NOT NULL REFERENCES clients(id),

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_blocks_on_cid
  ON blocks(cid);

CREATE TABLE uploads_blocks (
  upload_id TEXT NOT NULL REFERENCES uploads(id) ON DELETE CASCADE,
  block_id TEXT REFERENCES blocks(id),

  byte_offset INTEGER NOT NULL CHECK (byte_offset >= 0) CONSTRAINT byte_offset_positive,
  data_length INTEGER NOT NULL CHECK (data_length >= 0) CONSTRAINT data_length_positive,

  associated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_uploads_blocks_on_upload_id_block_id
  ON uploads_blocks(upload_id, block_id) WHERE block_id IS NOT NULL;

CREATE INDEX idx_uploads_blocks_on_upload_id
  ON uploads_blocks(upload_id) WHERE block_id IS NULL;
