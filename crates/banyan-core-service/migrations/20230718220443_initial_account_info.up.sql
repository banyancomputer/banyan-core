-- NEXT AUTH TABLES --

-- Migration for users table
CREATE TABLE users (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),
  name TEXT,
  email TEXT UNIQUE,
  email_verified TEXT,
  image TEXT
);

-- Migration for accounts table
CREATE TABLE accounts (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),
  userId TEXT NOT NULL,
  type TEXT NOT NULL,
  provider TEXT NOT NULL,
  providerAccountId TEXT NOT NULL,
  refresh_token TEXT,
  access_token TEXT,
  expires_at INTEGER,
  token_type TEXT,
  scope TEXT,
  id_token TEXT,
  session_state TEXT,
  FOREIGN KEY (userId) REFERENCES users(id)
);

-- Migration for sessions table
CREATE TABLE sessions (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),
  sessionToken TEXT UNIQUE,
  userId TEXT NOT NULL,
  expires TEXT NOT NULL,
  FOREIGN KEY (userId) REFERENCES users(id)
);

-- Migration for verification_tokens table
CREATE TABLE verification_tokens (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),
  token TEXT NOT NULL,
  identifier TEXT NOT NULL,
  expires TEXT NOT NULL
);

-- Migration for WebUI data

-- Migration for table specifying allow-listed emails for alpha
CREATE TABLE allowed_emails (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),
  email TEXT NOT NULL UNIQUE
);

-- Mggration for Escrowed Devices
CREATE TABLE escrowed_devices (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  account_id TEXT NOT NULL
    REFERENCES accounts(id)
    ON DELETE CASCADE,

  api_key_pem TEXT NOT NULL,
  encryption_key_pem TEXT NOT NULL,
  wrapped_api_key TEXT NOT NULL,
  wrapped_encryption_key TEXT NOT NULL,
  pass_key_salt TEXT NOT NULL,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_escrowed_device_keys_on_unique_account_id
  ON escrowed_devices(account_id);

-- Businesss Logic Tables

-- Migrations for Device API keys
CREATE TABLE device_api_keys (
  -- Dirty hack to generate UUIDs
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)),

  account_id TEXT NOT NULL
    REFERENCES accounts(id)
    ON DELETE CASCADE,

  fingerprint VARCHAR(50) NOT NULL,
  pem TEXT NOT NULL,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_device_api_keys_on_unique_fingerprint
  ON device_api_keys(fingerprint);

-- Migrations for Buckets
CREATE TABLE buckets (
  -- Dirty hack to generate UUIDs
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)),

  account_id TEXT NOT NULL
    REFERENCES accounts(id)
    ON DELETE CASCADE,

  name VARCHAR(128) NOT NULL,

  -- TODO: Make this an enum
  type VARCHAR(32) NOT NULL,

  -- TODO: Make this an enum
  storage_class VARCHAR(32) NOT NULL
);

CREATE UNIQUE INDEX idx_buckets_on_unique_account_id_and_name
  ON buckets(account_id, name);

-- Migrations for Bucket Keys
CREATE TABLE bucket_keys (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)),
  bucket_id TEXT NOT NULL
    REFERENCES buckets(id)
    ON DELETE CASCADE,
  pem TEXT NOT NULL,
  fingerprint TEXT NOT NULL,
  approved BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX idx_bucket_keys_on_bucket_id
  ON bucket_keys(bucket_id);

-- Migrations for Metadata
CREATE TABLE metadata (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)),

  bucket_id TEXT NOT NULL
    REFERENCES buckets(id)
    ON DELETE CASCADE,

  -- Description of the data
  -- The root CID of this version of the bucket
  root_cid TEXT NOT NULL,
  -- The CID of the metadata for this version of the bucket
  metadata_cid TEXT NOT NULL,

  -- What the client states as their expected data size on pushing metadata
  expected_data_size INTEGER NOT NULL,
  -- The size of the data pointed at by the root CID
  data_size INTEGER,

  -- Description of the metadata CAR file
  metadata_size INTEGER,
  metadata_hash TEXT,

  -- The state of the metadata
  -- TODO: Make this an enum
  state VARCHAR(32) NOT NULL,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_metadata_on_bucket_id
  ON metadata(bucket_id);

-- Migrations for Snapshots
CREATE TABLE snapshots (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)),

  metadata_id TEXT NOT NULL
    REFERENCES metadata(id)
    ON DELETE CASCADE,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_snapshots_on_unique_metadata_id
  ON snapshots(metadata_id);
   
-- Migration for Storage Hosts
CREATE TABLE storage_hosts (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)),

  -- Friendly name for the host
  name VARCHAR(128) NOT NULL UNIQUE,

  -- The host's url
  url TEXT NOT NULL,

  -- The host's currently used storage capacity (in bytes)
  used_storage INTEGER NOT NULL,

  -- The host's available storage capacity (in bytes)
  available_storage INTEGER NOT NULL,

  -- The fingerprint of the host's public key
  fingerprint VARCHAR(50) NOT NULL,

  -- The host's public key (PEM format)
  pem TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_storage_hosts_on_unique_name
  ON storage_hosts(name);

CREATE TABLE storage_grants (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)),

  storage_host_id TEXT NOT NULL REFERENCES storage_hosts(id),
  account_id TEXT NOT NULL REFERENCES accounts(id),
  authorized_amount INTEGER NOT NULL DEFAULT 0,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  redeemed_at TIMESTAMP
);

CREATE TABLE storage_hosts_metadatas_storage_grants (
  storage_host_id TEXT NOT NULL REFERENCES storage_hosts(id),
  metadata_id TEXT NOT NULL REFERENCES metadata(id),
  storage_grant_id TEXT REFERENCES storage_grants(id)
);
