CREATE TABLE accounts (
  -- Dirty hack to generate UUIDs
  id TEXT PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)),

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE device_api_keys (
  -- Dirty hack to generate UUIDs
  id TEXT PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)),

  account_id TEXT NOT NULL
    REFERENCES accounts(id)
    ON DELETE CASCADE,

  fingerprint VARCHAR(50) NOT NULL,
  public_key TEXT NOT NULL,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_device_api_keys_on_unique_fingerprint
  ON device_api_keys(fingerprint);

CREATE TABLE buckets (
  -- Dirty hack to generate UUIDs
  id TEXT PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)),

  account_id TEXT NOT NULL
    REFERENCES accounts(id)
    ON DELETE CASCADE,

  friendly_name VARCHAR(128),
  type VARCHAR(32) NOT NULL
);

CREATE UNIQUE INDEX idx_buckets_on_unique_account_id_friendly_name
  ON buckets(account_id, friendly_name);

CREATE TABLE bucket_keys (
  id TEXT PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)),

  bucket_id TEXT NOT NULL
    REFERENCES buckets(id)
    ON DELETE CASCADE,

  approved BOOLEAN NOT NULL DEFAULT false
);
