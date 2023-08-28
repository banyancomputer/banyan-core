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

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
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

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
);

CREATE UNIQUE INDEX idx_storage_grants_on_remote_grant_id
  ON storage_grants(remote_grant_id);
