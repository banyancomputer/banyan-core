-- Tally of how much storage has already been 'reserved' for client usage
ALTER TABLE storage_hosts ADD COLUMN reserved_storage INTEGER NOT NULL DEFAULT 0; 
-- This accounts for the history of the db but must be updated
UPDATE storage_hosts SET reserved_storage = (
	SELECT SUM(sg.authorized_amount)
	FROM storage_hosts sh
	INNER JOIN storage_grants AS sg 
	WHERE sg.storage_host_id = sh.id 
);
