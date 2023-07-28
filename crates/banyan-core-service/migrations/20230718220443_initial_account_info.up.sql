-- NEXT AUTH TABLES --

-- Migration for users table
CREATE TABLE users (
  id TEXT PRIMARY KEY DEFAULT (
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
  id TEXT PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),
  --- The escrowed device blob for the user's web device (contains encrypted private keys)
  escrowed_device_blob TEXT,
  --- The public encryption key for the escrowed device
  encryption_key_pem TEXT,
  --- The public signing key for the escrowed device
  api_key_pem TEXT,
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
  id TEXT PRIMARY KEY DEFAULT (
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
  id TEXT PRIMARY KEY DEFAULT (
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

-- Migration for table specifying allow-listed emails for alpha
CREATE TABLE allowed_emails (
  id TEXT PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),
  email TEXT NOT NULL UNIQUE
);

-- Device API keys
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
  pem TEXT NOT NULL,

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
