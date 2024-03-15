ALTER TABLE block_locations ADD COLUMN stored_at TIMESTAMP;
UPDATE block_locations SET stored_at = associated_at WHERE stored_at IS NULL;
