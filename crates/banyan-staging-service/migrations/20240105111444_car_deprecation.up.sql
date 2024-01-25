ALTER TABLE uploads RENAME COLUMN file_path TO base_path;
ALTER TABLE uploads_blocks ADD COLUMN car_offset INTEGER;
UPDATE uploads_blocks SET car_offset = byte_offset;
ALTER TABLE uploads_blocks DROP byte_offset;

-- ALTER TABLE uploads ADD COLUMN completed BOOLEAN;

-- upload.rs  car .bin (car_offset)

-- base_ath
-- /uploads/metadata_id.car


-- base_path new:
-- /uploads/upload_id/

--{cid}.bin 
