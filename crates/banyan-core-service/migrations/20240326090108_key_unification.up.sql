-- Unify Device and API keys

-- Part 1: Modify api keys
ALTER TABLE device_api_keys 
    -- rename table
    RENAME TO api_keys;

ALTER TABLE api_keys
    -- add name now that there will be many per user
    ADD COLUMN name TEXT NOT NULL;
ALTER TABLE api_keys
    -- add API access as column
    ADD COLUMN api_access BOOLEAN NOT NULL DEFAULT FALSE;

-- any existing api keys should be grandfathered in
UPDATE api_keys
    SET name = "Owner",
    api_access = TRUE;

-- this is pretty similar to the way bucket_keys currently works
CREATE TABLE api_key_access (
    id TEXT NOT NULL, -- Same as other instances
    api_key_id TEXT NOT NULL
        REFERENCES api_keys(id)
        ON DELETE CASCADE,
    bucket_id TEXT NOT NULL
        REFERENCES buckets(id)
        ON DELETE CASCADE,
    -- rather than simple y/n approval, let's include ['pending', 'approved', 'revoked'] as valid 
    state TEXT NOT NULL
         CHECK (state IN ('pending', 'approved', 'revoked'))
         DEFAULT 'pending'
);
