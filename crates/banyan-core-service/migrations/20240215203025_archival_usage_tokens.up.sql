-- Add the archival token column
ALTER TABLE subscriptions ADD COLUMN included_archival INTEGER NOT NULL DEFAULT 0; 

-- Track earned, consumed, and total available archival tokens
-- ALTER TABLE users ADD COLUMN earned_tokens INTEGER NOT NULL DEFAULT 0;
-- ALTER TABLE users ADD COLUMN consumed_tokens INTEGER NOT NULL DEFAULT 0;
-- ALTER TABLE users ADD COLUMN limit_tokens INTEGER NOT NULL;
-- ALTER TABLE snapshots ADD COLUMN consumed_tokens INTEGER NOT NULL;
