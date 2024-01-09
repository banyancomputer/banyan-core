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

CREATE TABLE snapshot_segments
(
    id          TEXT      NOT NULL PRIMARY KEY
        DEFAULT (
            lower(hex(randomblob(4))) || '-' ||
            lower(hex(randomblob(2))) || '-4' ||
            substr(lower(hex(randomblob(2))), 2) || '-a' ||
            substr(lower(hex(randomblob(2))), 2) || '-6' ||
            substr(lower(hex(randomblob(6))), 2)
            ),

    deal_id     TEXT      NOT NULL REFERENCES deals (id) ON DELETE CASCADE,
    size        INTEGER   NOT NULL,

    created_at  TIMESTAMP NOT NULL
        DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP NOT NULL
        DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE snapshot_segment_associations
(
    snapshot_id TEXT NOT NULL REFERENCES snapshots (id) ON DELETE CASCADE,
    segment_id  TEXT NOT NULL REFERENCES snapshot_segments (id) ON DELETE CASCADE,

    PRIMARY KEY (snapshot_id, segment_id)
);

CREATE TRIGGER update_deals_timestamp
    AFTER UPDATE
    ON deals
BEGIN
    UPDATE deals SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER update_snapshot_segments_timestamp
    AFTER UPDATE
    ON snapshot_segments
BEGIN
    UPDATE snapshot_segments SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
