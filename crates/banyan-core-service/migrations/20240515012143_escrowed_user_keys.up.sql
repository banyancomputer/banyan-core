-- Update escrowed_devices to match this new paradigm
ALTER TABLE escrowed_devices RENAME TO escrowed_user_keys;
ALTER TABLE escrowed_user_keys RENAME COLUMN api_public_key_pem TO public_key;
ALTER TABLE escrowed_user_keys DROP COLUMN encryption_public_key_pem;

