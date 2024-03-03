CREATE TABLE metrics_storage
(
    user_id                TEXT      NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    hot_storage_bytes      INTEGER   NOT NULL DEFAULT 0,
    archival_storage_bytes INTEGER   NOT NULL DEFAULT 0,
    storage_host_id        TEXT      NOT NULL REFERENCES storage_hosts (id) ON DELETE CASCADE,
    slot                   TIMESTAMP NOT NULL
);

CREATE INDEX idx_metrics_storage_user_id ON metrics_storage (user_id);
CREATE INDEX idx_metrics_storage_storage_host_id ON metrics_storage (storage_host_id);
CREATE INDEX idx_metrics_storage_slot ON metrics_storage (slot);
CREATE UNIQUE INDEX idx_metrics_storage_slot_user_id_storage_host_id ON metrics_storage (slot, user_id, storage_host_id);
