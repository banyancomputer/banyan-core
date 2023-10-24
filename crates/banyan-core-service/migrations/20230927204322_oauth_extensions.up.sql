CREATE TABLE rust_users (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  email TEXT NOT NULL,
  display_name TEXT NOT NULL,
  locale TEXT,
  profile_image TEXT,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_unique_rust_users_on_email ON
  rust_users(email);

CREATE TABLE oauth_state (
  provider TEXT NOT NULL,
  csrf_secret TEXT NOT NULL,
  pkce_verifier_secret TEXT NOT NULL,

  next_url TEXT,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_unique_oauth_state_on_provider_csrf_secret
  ON oauth_state(provider, csrf_secret);

CREATE TABLE rust_sessions (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  user_id TEXT NOT NULL REFERENCES rust_users(id),
  provider TEXT NOT NULL,

  client_ip TEXT,
  user_agent TEXT,

  access_token TEXT NOT NULL,
  access_expires_at TIMESTAMP,
  refresh_token TEXT,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMP NOT NULL
);
