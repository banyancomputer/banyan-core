-- Tally of how much storage has already been 'reserved' for client usage
ALTER TABLE storage_hosts ADD COLUMN reserved_storage INTEGER NOT NULL DEFAULT 0; 
