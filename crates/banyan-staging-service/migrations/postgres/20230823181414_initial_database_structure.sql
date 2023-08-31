CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE clients (
  id UUID DEFAULT uuid_generate_v4() NOT NULL PRIMARY KEY,

  platform_id UUID NOT NULL,
  fingerprint VARCHAR(64) NOT NULL,
  public_key TEXT NOT NULL,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_clients_on_fingerprint
  ON clients(fingerprint);

CREATE TABLE storage_grants (
  id UUID DEFAULT uuid_generate_v4() NOT NULL PRIMARY KEY,

  client_id UUID NOT NULL REFERENCES clients(id),
  allowed_storage INTEGER NOT NULL DEFAULT 0 CHECK (allowed_storage > 0),

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_storage_grants_on_created_at
  ON storage_grants(created_at);

CREATE TABLE uploads (
  id UUID DEFAULT uuid_generate_v4() NOT NULL PRIMARY KEY,

  client_id UUID NOT NULL REFERENCES clients(id),
  metadata_id UUID NOT NULL,

  reported_size INTEGER NOT NULL CHECK (reported_size >= 0),
  final_size INTEGER NOT NULL DEFAULT 0 CHECK (reported_size >= 0),

  file_path VARCHAR(128) NOT NULL,
  state VARCHAR(32) NOT NULL CHECK (state IN ('started', 'indexing', 'complete', 'failed')),
  integrity_hash VARCHAR(32),

  started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  finished_at TIMESTAMP
);

CREATE INDEX idx_uploads_on_client_id
  ON uploads(client_id);
CREATE UNIQUE INDEX idx_uploads_on_metadata_id
  ON uploads(metadata_id);

CREATE TABLE blocks (
  id UUID DEFAULT uuid_generate_v4() NOT NULL PRIMARY KEY,

  cid VARCHAR(64) NOT NULL,
  owner_id UUID NOT NULL REFERENCES clients(id),

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_blocks_on_cid
  ON blocks(cid);

CREATE TABLE uploads_blocks (
  upload_id UUID NOT NULL REFERENCES uploads(id) ON DELETE CASCADE,
  block_id UUID NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,

  byte_offset INTEGER NOT NULL CHECK (byte_offset >= 0),
  data_length INTEGER NOT NULL CHECK (data_length >= 0),

  associated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_uploads_blocks_on_upload_id_block_id
  ON uploads_blocks(upload_id, block_id);
