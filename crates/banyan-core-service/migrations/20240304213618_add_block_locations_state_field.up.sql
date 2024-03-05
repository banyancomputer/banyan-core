ALTER TABLE block_locations
    ADD COLUMN state VARCHAR(32) NOT NULL CHECK (
        state IN ('sync_required', 'staged', 'stable')
        ) DEFAULT 'stable';

UPDATE block_locations
SET state = 'sync_required'
WHERE state = 'stable' AND block_locations.storage_host_id == (SELECT id FROM storage_hosts WHERE staging IS TRUE);
