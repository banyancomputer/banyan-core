-- Unify Device and API keys

-- Part 1: Modify api keys
ALTER TABLE device_api_keys 
    -- rename table
    RENAME TO user_keys;

ALTER TABLE user_keys
    -- add name now that there will be many per user
    ADD COLUMN name TEXT NOT NULL;

ALTER TABLE user_keys
	RENAME COLUMN pem TO public_key;

ALTER TABLE user_keys
    -- add API access as column
    ADD COLUMN api_access BOOLEAN NOT NULL DEFAULT FALSE;

-- any existing api keys should be grandfathered in
UPDATE user_keys
    SET name = "Owner",
    api_access = TRUE;

-- this is pretty similar to the way bucket_keys currently works
CREATE TABLE bucket_access (
    user_key_id TEXT NOT NULL
        REFERENCES user_keys(id)
        ON DELETE CASCADE,
    bucket_id TEXT NOT NULL
        REFERENCES buckets(id)
        ON DELETE CASCADE,
	approved BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE UNIQUE INDEX idx_api_access_on_buckets
  ON bucket_access(user_key_id, bucket_id);

-- Part 2: Modify Bucket Keys
-- These will default to being keys without API access
INSERT INTO user_keys(id, user_id, fingerprint, public_key, name)
    SELECT 
		bk.id,
        u.id, 
        bk.fingerprint, 
        bk.pem, 
        ("Inherited key from " || b.name) 
    FROM bucket_keys AS bk
    JOIN buckets AS b ON bk.bucket_id = b.id
    JOIN users AS u ON b.user_id = u.id
;

INSERT INTO bucket_access(user_key_id, bucket_id, approved) 
    SELECT
        ak.id,
        bk.bucket_id,
		bk.approved
    FROM bucket_keys AS bk
    JOIN user_keys AS ak ON ak.fingerprint = bk.fingerprint
;

-- Scary! ^w^
DROP TABLE bucket_keys;
