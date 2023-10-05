-- We're only using OAuth2 w/ Google. All accounts are inherently verified by
-- logging in.
ALTER TABLE users DROP COLUMN email_verified;

-- Better naming on a couple of the existing columns...
ALTER TABLE users RENAME COLUMN image TO profile_image_url;
ALTER TABLE users RENAME COLUMN name TO account_name;

-- Changing a column that has foreign keys referencing it is much harder...

CREATE TABLE new_accounts(
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

  provider_account_id TEXT NOT NULL,

  refresh_token TEXT NOT NULL,
  access_token TEXT NOT NULL,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMP NOT NULL
);

INSERT INTO new_accounts (id, user_id, provider_account_id, refresh_token, access_token, expires_at)
  SELECT id, userId, providerAccountId, refresh_token, access_token, DATETIME(expires_at, 'unixepoch') FROM accounts
    -- We can only migrate accounts to this new table that meet our
    -- requirements, this should be all of them but add a guard just in case.
    -- We don't need to specify columns that previously had a NOT NULL
    -- constraint.
    WHERE
      accounts.refresh_token IS NOT NULL AND
      accounts.access_token IS NOT NULL AND
      accounts.expires_at IS NOT NULL;

-- TODO: Need to do this for all associations and likely I don't have handy
-- names on the constraints, probably should be the user_id anyway... Ugh
-- that's going to be an annoying alteration...
--ALTER TABLE escrowed_devices DROP FOREIGN KEY escrowed_devices_ibfk_1;

--ALTER TABLE accounts RENAME TO old_accounts;
--ALTER TABLE new_accounts RENAME TO accounts;
--
--ALTER TABLE escrowed_devices
--  ADD CONSTRAINT fk_accounts
--  FOREIGN KEY (account_id) REFERENCES accounts(id)
--  ON DELETE CASCADE;
--
--DROP TABLE old_accounts;
