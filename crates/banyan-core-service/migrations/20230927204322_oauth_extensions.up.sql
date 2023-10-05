-- !!! We're missing a not null constraint on email and that can't be easily
-- fixed. The table needs to be renamed, recreated correctly, and populated
-- with the data from the old table... but there are foreign key constraints
-- that all need to be disconnected and reconnected to fix it.

-- We need to keep around the old columns until we've fully migrated over...
ALTER TABLE users ADD COLUMN display_name VARCHAR(128);
ALTER TABLE users ADD COLUMN locale VARCHAR(256);
ALTER TABLE users ADD COLUMN profile_image VARCHAR(256);
ALTER TABLE users ADD COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- ...but we need to remove the constraints on the existing columns...
ALTER TABLE sessions ALTER COLUMN sessionToken DROP UNIQUE;
ALTER TABLE sessions ALTER COLUMN userId DROP NOT NULL;
ALTER TABLE sessions ALTER COLUMN expires DROP NOT NULL;

-- ...and this column is especially annoying, anonymous FK aren't allowed to be
-- NULL and can't be referenced. Changing them requires copying everything into
-- a new column defined in a way that can be adjusted.
ALTER TABLE sessions ADD COLUMN user_id TEXT REFERENCES users(id) DEFAULT userId;
CREATE UNIQUE INDEX idx_unique_sessions_on_user_id
  ON sessions(user_id);

-- ...remove and create the table for nextJS
ALTER TABLE sessions DROP COLUMN userId;
ALTER TABLE sessions ADD COLUMN userId TEXT REFERENCE users(id) NOT NULL DEFAULT user_id;
CREATE UNIQUE INDEX idx_unique_sessions_on_userId
  ON sessions(userId);

-- ...add the extra columns we want
ALTER TABLE sessions ADD COLUMN client_ip VARCHAR(64);
ALTER TABLE sessions ADD COLUMN user_agent VARCHAR(128);
ALTER TABLE sessions ADD COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE sessions ADD COLUMN expires_at TIMESTAMP NOT NULL DEFAULT (DATETIME('now', '+28 days'));

CREATE TABLE oauth_state (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  provider VARCHAR(32) NOT NULL,
  csrf_secret TEXT NOT NULL,
  pkce_verifier_secret TEXT NOT NULL,

  next_url VARCHAR(256),

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
