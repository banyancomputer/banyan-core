-- Delete the existing unique index
DROP INDEX idx_buckets_on_unique_user_id_and_name;
-- Create the new index which will replace it
-- Note that we IFNULl the deleted_at column to maintain the unique constraint
-- that non-deleted buckets must be unique across user_id and name.
CREATE UNIQUE INDEX idx_buckets_on_unique_user_id_and_name_and_deleted_at
  ON buckets(user_id, name, IFNULL(deleted_at, 0));

