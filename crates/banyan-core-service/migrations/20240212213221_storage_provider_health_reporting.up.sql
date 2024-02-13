-- Last we heard from the host
ALTER TABLE storage_hosts ADD COLUMN last_seen_at TIMESTAMP;
-- Version 
ALTER TABLE storage_hosts ADD COLUMN current_version TEXT;
