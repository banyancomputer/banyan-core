ALTER TABLE buckets ADD COLUMN deleted_at TIMESTAMP;
ALTER TABLE buckets ADD COLUMN updated_at TIMESTAMP;

UPDATE buckets SET updated_at = created_at;
