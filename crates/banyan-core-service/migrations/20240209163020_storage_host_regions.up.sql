-- Add a region column so storage hosts can be selected along this preference 
ALTER TABLE storage_hosts ADD COLUMN region TEXT NOT NULL DEFAULT 'unspecified';
-- Add a regional preference column to the users table so we can factor in
ALTER TABLE users ADD COLUMN region_preference TEXT;
