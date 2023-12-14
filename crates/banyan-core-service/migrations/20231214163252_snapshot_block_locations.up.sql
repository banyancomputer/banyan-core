CREATE TABLE snapshot_block_locations (
  snapshot_id TEXT NOT NULL REFERENCES snapshots(id),
  block_id TEXT NOT NULL REFERENCES blocks(id)
);
