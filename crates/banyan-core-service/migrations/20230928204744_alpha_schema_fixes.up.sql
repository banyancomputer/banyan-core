-- We need to record and return how large the snapshot is
ALTER TABLE snapshots ADD COLUMN size INTEGER NOT NULL;

-- Once a snapshot is created we need to assign it to a storage_host provider
-- for eventual sealing. This table hold the association of that assignment and
-- our on going monitoring of state around that process.
CREATE TABLE snapshots_storage_hosts(
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  snapshot_id TEXT NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
  storage_host_id TEXT NOT NULL REFERENCES storage_hosts(id) ON DELETE CASCADE,

  assigned_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  due_by TIMESTAMP NOT NULL DEFAULT (DATETIME('now', '+1 day')),

  status VARCHAR(32) NOT NULL CHECK (status IN ('assigned', 'accepted', 'sealing', 'sealed', 'failed', 'expired')),

  -- This timestamp represents the time this association entered its 'final'
  -- state which will be one of 'sealed', 'failed', or 'expired' and indicates
  -- when it reached that state.
  finalized_at TIMESTAMP
);

CREATE INDEX idx_snapshots_storage_hosts_on_status
  ON snapshots_storage_hosts(status);

CREATE UNIQUE INDEX idx_snapshots_storage_hosts_on_snapshot_id_storage_host_id
  ON snapshots_storage_hosts(snapshot_id, storage_host_id)
  WHERE status NOT IN ('failed', 'expired');
