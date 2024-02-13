use std::time::Duration;

use banyan_traffic_counter::body::{RequestInfo, ResponseInfo};
use banyan_traffic_counter::on_response_end::OnResponseEnd;
use time::OffsetDateTime;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::database::models::BandwidthMetrics;
use crate::database::Database;

const ONE_HOUR_DURATION: Duration = Duration::from_secs(60 * 60);

#[derive(Clone)]
pub struct TrafficReporter {
    database: Database,
    tx: UnboundedSender<BandwidthMetrics>,
}

impl<B> OnResponseEnd<B> for TrafficReporter {
    fn on_response_end(&self, req_info: &RequestInfo, res_info: &ResponseInfo) {
        let ingress = (req_info.header_bytes + req_info.body_bytes) as i64;
        let egress = (res_info.header_bytes + res_info.body_bytes) as i64;
        let tx = self.tx.clone();
        let user_id = match &res_info.traffic_counter_handle.user_id {
            Some(user_id) => user_id.clone(),
            None => return,
        };
        if let Err(SendError(err)) = tx.send(BandwidthMetrics {
            user_id: user_id.clone(),
            ingress,
            egress,
        }) {
            tracing::error!(
                "could not send metrics to db for user {} err {:?}",
                user_id,
                err
            );
        }
    }
}
impl TrafficReporter {
    pub fn new(database: Database) -> Self {
        let (tx, rx) = unbounded_channel();
        let reporter = Self { database, tx };
        reporter.start_metrics_flush_task(rx);
        reporter
    }

    fn start_metrics_flush_task(&self, mut rx: UnboundedReceiver<BandwidthMetrics>) {
        let reporter = self.clone();
        tokio::spawn(async move {
            while let Some(user_metrics) = rx.recv().await {
                if let Err(e) = user_metrics
                    .save(&reporter.database, OffsetDateTime::now_utc())
                    .await
                {
                    tracing::error!(
                        "failed to save metrics for user: {} err: {}",
                        user_metrics.user_id.as_str(),
                        e
                    );
                }
            }
        });
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReportTrafficError {
    #[error("failed to send request: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("failed to send request: {0}")]
    JWTError(#[from] jwt_simple::Error),
}
