-- Add a region column so storage hosts can be selected along this preference 
ALTER TABLE storage_hosts ADD COLUMN region TEXT NOT NULL DEFAULT 'unspecified';
