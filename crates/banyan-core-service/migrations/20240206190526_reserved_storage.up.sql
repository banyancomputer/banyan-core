-- Tally of how much storage has already been 'reserved' for client usage
ALTER TABLE storage_hosts ADD COLUMN reserved_storage INTEGER NOT NULL DEFAULT 0; 
-- This accounts for the history of the db but must be updated
UPDATE storage_hosts SET reserved_storage = (
	SELECT SUM(sg.authorized_amount)
	FROM storage_hosts sh
	-- Ensure to only select from a subset of storage_grants where
	-- the user_id is unique and the redemption time is most recent 
	INNER JOIN (
        SELECT user_id, storage_host_id, MAX(redeemed_at) as redeemed_at, authorized_amount 
        FROM storage_grants
        GROUP BY user_id
	) AS sg 
	WHERE sg.storage_host_id = sh.id 
	AND sg.redeemed_at <> NULL
	ORDER BY sg.redeemed_at
);

