-- NEXT AUTH TABLES --

-- Users table for all auth providers
CREATE TABLE users (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),


  -- User Email Information
  email TEXT NOT NULL,
  verified_email BOOLEAN NOT NULL DEFAULT false,

  -- User Profile Information
  display_name TEXT NOT NULL,
  locale TEXT,
  profile_image TEXT,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_unique_users_on_email ON
  users(email);

-- OAuth Provider Accounts
CREATE TABLE oauth_provider_accounts (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  user_id TEXT NOT NULL
    REFERENCES users(id)
    ON DELETE CASCADE,

  -- Name of the provider
  provider TEXT NOT NULL,
  -- Account ID on the provider
  provider_id TEXT NOT NULL,
  -- TODO: Is this required? What does it mean?
  -- provider_email TEXT NOT NULL,

  associated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_unique_oauth_provider_accounts_on_provider_provider_id
  ON oauth_provider_accounts(provider, provider_id);
-- TODO: Figure out if this is needed
-- CREATE UNIQUE INDEX idx_unique_oauth_provider_accounts_on_provider_provider_email
--   ON oauth_provider_accounts(provider, provider_email);

-- Table for managing OAuth login state
CREATE TABLE oauth_state (
  provider TEXT NOT NULL,
  csrf_secret TEXT NOT NULL,
  pkce_verifier_secret TEXT NOT NULL,

  next_url TEXT,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_unique_oauth_state_on_provider_csrf_secret
  ON oauth_state(provider, csrf_secret);

-- Table for managing OAuth Sessions
CREATE TABLE sessions (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  user_id TEXT NOT NULL REFERENCES users(id),
  provider TEXT NOT NULL,

  client_ip TEXT,
  user_agent TEXT,

  access_token TEXT NOT NULL,
  access_expires_at TIMESTAMP,
  refresh_token TEXT,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMP NOT NULL
);

-- Migration for WebUI data

-- Migration for Escrowed Devices
CREATE TABLE escrowed_devices (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  user_id TEXT NOT NULL
    REFERENCES users(id)
    ON DELETE CASCADE,

  api_public_key_pem TEXT NOT NULL,
  encryption_public_key_pem TEXT NOT NULL,
  encrypted_private_key_material TEXT NOT NULL,
  pass_key_salt TEXT NOT NULL,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_escrowed_device_keys_on_unique_user_id
  ON escrowed_devices(user_id);

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

  user_id TEXT NOT NULL
    REFERENCES users(id)
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

  user_id TEXT NOT NULL
    REFERENCES users(id)
    ON DELETE CASCADE,

  name VARCHAR(128) NOT NULL,

  -- TODO: Make this an enum
  type VARCHAR(32) NOT NULL,

  -- TODO: Make this an enum
  storage_class VARCHAR(32) NOT NULL

  -- todo: this needs created_at and updated_at fields
);

CREATE UNIQUE INDEX idx_buckets_on_unique_user_id_and_name
  ON buckets(user_id, name);

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

  -- todo: should have a user_id

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
  -- 'uploading' means the metadata is being uploaded the server
  -- 'upload_failed' means the metadata failed to upload to the server
  -- 'pending' means the metadata has been processed by the server and waiting conf by the staging server
  -- 'current' means the metadata has been fully processed and is the current version of the bucket
  -- 'outdated' means the metadata has been fully processed but is not the current version of the bucket
  -- 'deleted' means the metadata has been deleted
  state TEXT NOT NULL,

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

  state TEXT NOT NULL,
  size INTEGER,

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
  user_id TEXT NOT NULL REFERENCES users(id),
  authorized_amount INTEGER NOT NULL DEFAULT 0,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  redeemed_at TIMESTAMP
);

CREATE TABLE storage_hosts_metadatas_storage_grants (
  storage_host_id TEXT NOT NULL
    REFERENCES storage_hosts(id)
    ON DELETE CASCADE,

  metadata_id TEXT NOT NULL
    REFERENCES metadata(id)
    ON DELETE CASCADE,

  storage_grant_id TEXT NOT NULL
    REFERENCES storage_grants(id)
    ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_storage_hosts_metadatas_storage_grants_on_all
  ON storage_hosts_metadatas_storage_grants(storage_host_id, metadata_id, storage_grant_id);

CREATE TABLE snapshot_restore_requests (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)),

  user_id TEXT
    REFERENCES users(id)
    ON DELETE SET NULL,

  snapshot_id TEXT
    REFERENCES snapshots(id)
    ON DELETE SET NULL,

  storage_host_id TEXT
    REFERENCES storage_hosts(id)
    ON DELETE SET NULL,

  state TEXT NOT NULL,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  completed_at TIMESTAMP
);

CREATE UNIQUE INDEX idx_snapshot_restore_requests_on_snapshot_id_storage_host_id_state
  ON snapshot_restore_requests(snapshot_id, storage_host_id)
  WHERE snapshot_id IS NOT NULL
    AND storage_host_id IS NOT NULL
    AND user_id IS NOT NULL
    AND state IN ('pending', 'ready');
