ALTER TABLE uploads DROP COLUMN file_path;
ALTER TABLE blocks ADD COLUMN car_offset INTEGER CHECK (car_offset IS NULL OR car_offset >= 0);

INSERT INTO blocks(car_offset)
SELECT byte_offset as car_offset
FROM uploads_blocks 
JOIN blocks
WHERE blocks.id=uploads_blocks.block_id;

ALTER TABLE uploads_blocks DROP byte_offset;
