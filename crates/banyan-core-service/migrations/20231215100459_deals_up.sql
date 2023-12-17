-- Add up migration script here
CREATE TABLE deals
(
    id          TEXT      NOT NULL PRIMARY KEY
        DEFAULT (
            lower(hex(randomblob(4))) || '-' ||
            lower(hex(randomblob(2))) || '-4' ||
            substr(lower(hex(randomblob(2))), 2) || '-a' ||
            substr(lower(hex(randomblob(2))), 2) || '-6' ||
            substr(lower(hex(randomblob(6))), 2)
            ),

    state       TEXT      NOT NULL
        CHECK (state IN ('active', 'accepted', 'sealed', 'finalized', 'cancelled'))
        DEFAULT 'active',

    accepted_by TEXT REFERENCES storage_hosts (id),
    accepted_at TIMESTAMP,

    created_at  TIMESTAMP NOT NULL
        DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP NOT NULL
        DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER update_deals_timestamp
    AFTER UPDATE
    ON deals
BEGIN
    UPDATE deals SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

ALTER TABLE snapshots ADD COLUMN deal_id TEXT REFERENCES deals (id);



