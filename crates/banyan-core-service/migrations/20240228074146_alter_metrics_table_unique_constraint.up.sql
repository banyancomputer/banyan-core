DROP INDEX idx_metrics_traffic_slot_user_id;
CREATE UNIQUE INDEX idx_metrics_traffic_slot_user_id_storage_host_id ON metrics_traffic (slot, storage_host_id, user_id);
