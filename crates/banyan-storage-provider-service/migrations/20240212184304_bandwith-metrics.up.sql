CREATE TABLE bandwidth_metrics
(
    user_id    TEXT   NOT NULL,
    ingress    INTEGER  NOT NULL DEFAULT 0,
    egress     INTEGER  NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_metrics_traffic_user_id ON bandwidth_metrics (user_id);
CREATE INDEX idx_metrics_traffic_created_at ON bandwidth_metrics (created_at);
