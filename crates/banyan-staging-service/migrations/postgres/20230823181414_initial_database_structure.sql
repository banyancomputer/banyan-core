CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE clients (
  id UUID DEFAULT uuid_generate_v4() NOT NULL PRIMARY KEY,

  platform_id TEXT NOT NULL,
  fingerprint VARCHAR(64) NOT NULL,
  public_key TEXT NOT NULL,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
);

CREATE UNIQUE INDEX idx_clients_on_fingerprint
  ON clients(fingerprint);

CREATE TABLE storage_grants (
  id UUID DEFAULT uuid_generate_v4() NOT NULL PRIMARY KEY,

  client_id UUID NOT NULL REFERENCES clients(id),
  remote_grant_id UUID NOT NULL,
  allowed_storage INTEGER NOT NULL DEFAULT 0 CHECK (allowed_storage > 0),

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
);

CREATE UNIQUE INDEX idx_storage_grants_on_remote_grant_id
  ON storage_grants(remote_grant_id);
