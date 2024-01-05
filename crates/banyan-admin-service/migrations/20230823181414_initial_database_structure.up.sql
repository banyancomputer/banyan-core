-- NEXT AUTH TABLES --

-- Users table for all auth providers
CREATE TABLE users
(
    id              TEXT      NOT NULL PRIMARY KEY DEFAULT (
        lower(hex(randomblob(4))) || '-' ||
        lower(hex(randomblob(2))) || '-4' ||
        substr(lower(hex(randomblob(2))), 2) || '-a' ||
        substr(lower(hex(randomblob(2))), 2) || '-6' ||
        substr(lower(hex(randomblob(6))), 2)
        ),


    -- User Email Information
    email           TEXT      NOT NULL,
    verified_email  BOOLEAN   NOT NULL             DEFAULT false,

    -- User Profile Information
    display_name    TEXT      NOT NULL,
    locale          TEXT,
    profile_image   TEXT,

    created_at      TIMESTAMP NOT NULL             DEFAULT CURRENT_TIMESTAMP,
    accepted_tos_at TIMESTAMP
);

CREATE UNIQUE INDEX idx_unique_users_on_email ON
    users (email);

-- OAuth Provider Accounts
CREATE TABLE oauth_provider_accounts
(
    id            TEXT      NOT NULL PRIMARY KEY DEFAULT (
        lower(hex(randomblob(4))) || '-' ||
        lower(hex(randomblob(2))) || '-4' ||
        substr(lower(hex(randomblob(2))), 2) || '-a' ||
        substr(lower(hex(randomblob(2))), 2) || '-6' ||
        substr(lower(hex(randomblob(6))), 2)
        ),

    user_id       TEXT      NOT NULL
        REFERENCES users (id)
            ON DELETE CASCADE,

    -- Name of the provider
    provider      TEXT      NOT NULL,
    -- Account ID on the provider
    provider_id   TEXT      NOT NULL,
    -- TODO: Is this required? What does it mean?
    -- provider_email TEXT NOT NULL,

    associated_at TIMESTAMP NOT NULL             DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_unique_oauth_provider_accounts_on_provider_provider_id
    ON oauth_provider_accounts (provider, provider_id);
-- TODO: Figure out if this is needed
-- CREATE UNIQUE INDEX idx_unique_oauth_provider_accounts_on_provider_provider_email
--   ON oauth_provider_accounts(provider, provider_email);

-- Table for managing OAuth login state
CREATE TABLE oauth_state
(
    provider             TEXT      NOT NULL,
    csrf_secret          TEXT      NOT NULL,
    pkce_verifier_secret TEXT      NOT NULL,

    next_url             TEXT,

    created_at           TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_unique_oauth_state_on_provider_csrf_secret
    ON oauth_state (provider, csrf_secret);

-- Table for managing OAuth Sessions
CREATE TABLE sessions
(
    id                TEXT      NOT NULL PRIMARY KEY DEFAULT (
        lower(hex(randomblob(4))) || '-' ||
        lower(hex(randomblob(2))) || '-4' ||
        substr(lower(hex(randomblob(2))), 2) || '-a' ||
        substr(lower(hex(randomblob(2))), 2) || '-6' ||
        substr(lower(hex(randomblob(6))), 2)
        ),

    user_id           TEXT      NOT NULL REFERENCES users (id),
    provider          TEXT      NOT NULL,

    client_ip         TEXT,
    user_agent        TEXT,

    access_token      TEXT      NOT NULL,
    access_expires_at TIMESTAMP,
    refresh_token     TEXT,

    created_at        TIMESTAMP NOT NULL             DEFAULT CURRENT_TIMESTAMP,
    expires_at        TIMESTAMP NOT NULL
);
