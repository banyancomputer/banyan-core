-- Add up migration script here
ALTER TABLE storage_hosts ADD COLUMN continent TEXT NOT NULL DEFAULT 'unspecified';
