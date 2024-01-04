-- Add up migration script here
ALTER TABLE uploads RENAME COLUMN file_path to blocks_path;
