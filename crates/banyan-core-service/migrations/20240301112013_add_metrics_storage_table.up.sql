CREATE TABLE user_total_consumption
(
    user_id                TEXT      NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    hot_storage_bytes      INTEGER   NOT NULL DEFAULT 0,
    archival_storage_bytes INTEGER   NOT NULL DEFAULT 0,
    slot                   TIMESTAMP NOT NULL
);

CREATE INDEX idx_user_total_consumption_user_id ON user_total_consumption (user_id);
CREATE INDEX idx_user_total_consumption_slot ON user_total_consumption (slot);
CREATE UNIQUE INDEX idx_user_total_consumption_slot_user_id ON user_total_consumption (slot, user_id);


CREATE TABLE storage_host_total_consumption
(
    storage_host_id TEXT      NOT NULL REFERENCES storage_hosts (id) ON DELETE CASCADE,
    storage_bytes   INTEGER   NOT NULL DEFAULT 0,
    slot            TIMESTAMP NOT NULL
);

CREATE INDEX idx_storage_host_total_consumption_storage_host_id ON storage_host_total_consumption (storage_host_id);
CREATE INDEX idx_storage_host_total_consumption_slot ON storage_host_total_consumption (slot);
CREATE UNIQUE INDEX idx_storage_host_total_consumption_slot_user_id ON storage_host_total_consumption (slot, storage_host_id);
