CREATE TABLE metrics_traffic
(
    user_id         TEXT      NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    ingress         INTEGER   NOT NULL DEFAULT 0,
    egress          INTEGER   NOT NULL DEFAULT 0,
    storage_host_id TEXT      NOT NULL REFERENCES storage_hosts (id) ON DELETE CASCADE,
    slot            TIMESTAMP NOT NULL
);

CREATE INDEX idx_metrics_traffic_user_id ON metrics_traffic (user_id);
CREATE INDEX idx_metrics_traffic_storage_host_id ON metrics_traffic (storage_host_id);
CREATE INDEX idx_metrics_traffic_slot ON metrics_traffic (slot);
CREATE UNIQUE INDEX idx_metrics_traffic_slot_user_id ON metrics_traffic (slot, user_id);
