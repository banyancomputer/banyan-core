-- Add the archival token column
ALTER TABLE subscriptions ADD COLUMN included_archival INTEGER NOT NULL DEFAULT 0; 
-- Update existing subscriptions to reflect the appropriate included amount
UPDATE subscriptions SET included_archival = 
(
	CASE 
		WHEN service_key = "starter"  THEN 10
		WHEN service_key = "lite"     THEN 150
		WHEN service_key = "business" THEN 3072
		ELSE 0
	END
); 
-- Track earned, consumed, and total available archival tokens
-- ALTER TABLE users ADD COLUMN earned_tokens INTEGER NOT NULL DEFAULT 0;
-- ALTER TABLE users ADD COLUMN consumed_tokens INTEGER NOT NULL DEFAULT 0;
-- ALTER TABLE users ADD COLUMN limit_tokens INTEGER NOT NULL;
-- ALTER TABLE snapshots ADD COLUMN consumed_tokens INTEGER NOT NULL;