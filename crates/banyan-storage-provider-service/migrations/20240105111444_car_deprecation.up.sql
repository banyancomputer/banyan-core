-- This is now the base path, might change in the future, and single blocks go in dirs
ALTER TABLE uploads RENAME COLUMN file_path TO base_path;
-- Rename the byte_offset column to car_offset and remove the old constraint in favor of a NULL-friendly one
ALTER TABLE uploads_blocks ADD COLUMN car_offset INTEGER CHECK (car_offset >= 0) constraint offset_non_negative;
UPDATE uploads_blocks SET car_offset = byte_offset;
ALTER TABLE uploads_blocks DROP byte_offset;