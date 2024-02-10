CREATE TABLE metrics_traffic
(
    user_id    TEXT   NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    ingress    INTEGER  NOT NULL DEFAULT 0,
    egress     INTEGER  NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_metrics_traffic_user_id ON metrics_traffic (user_id);
CREATE INDEX idx_metrics_traffic_timestamp ON metrics_traffic (created_at);

