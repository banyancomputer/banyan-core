use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use banyan_traffic_counter::body::{RequestInfo, ResponseInfo};
use banyan_traffic_counter::on_response_end::OnResponseEnd;
use http::{HeaderMap, HeaderValue};
use jwt_simple::algorithms::ECDSAP384KeyPairLike;
use jwt_simple::claims::Claims;
use jwt_simple::prelude::*;
use reqwest::Client;
use tokio::time::interval;
use url::Url;

use crate::app::AppState;
use crate::utils::SigningKey;

const ONE_HOUR_DURATION: Duration = Duration::from_secs(60 * 60);

#[derive(Clone)]
pub struct TrafficReporter {
    app: AppState,
    user_traffic: Arc<RwLock<HashMap<String, (usize, usize)>>>,
}

impl<B> OnResponseEnd<B> for TrafficReporter {
    fn on_response_end(&self, req_info: &RequestInfo, res_info: &ResponseInfo) {
        let user_id = res_info.session.user_id.lock().unwrap();
        let user_id = match &*user_id {
            Some(key_id) => key_id,
            None => return,
        };
        let ingress = req_info.header_bytes + req_info.body_bytes;
        let egress = res_info.header_bytes + res_info.body_bytes;
        self.log_traffic(user_id, ingress, egress);
    }
}
impl TrafficReporter {
    pub fn new(app: AppState) -> Self {
        let reporter = Self {
            app,
            user_traffic: Arc::new(RwLock::new(HashMap::new())),
        };
        reporter.start_metrics_flush_task();
        reporter
    }

    fn platform_hostname(&self) -> Url {
        self.app.platform_hostname()
    }
    fn service_name(&self) -> &str {
        self.app.service_name()
    }
    fn platform_name(&self) -> &str {
        self.app.platform_name()
    }
    fn service_signing_key(&self) -> SigningKey {
        self.app.secrets().service_signing_key()
    }

    fn log_traffic(&self, user_id: &String, ingress: usize, egress: usize) {
        let mut user_traffic = self.user_traffic.write().unwrap();
        let (current_ingress, current_egress) = *user_traffic.get(user_id).unwrap_or(&(0, 0));
        user_traffic.insert(
            user_id.clone(),
            (current_ingress + ingress, current_egress + egress),
        );
    }

    fn start_metrics_flush_task(&self) {
        let reporter = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(ONE_HOUR_DURATION);
            loop {
                interval.tick().await;
                let users = reporter
                    .user_traffic
                    .read()
                    .unwrap()
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>();
                for user_id in users {
                    if let Err(e) = reporter.flush_traffic_metrics(user_id).await {
                        tracing::error!("Failed to flush traffic metrics: {}", e);
                    }
                }
            }
        });
    }

    async fn flush_traffic_metrics(&self, user_id: String) -> Result<(), ReportTrafficError> {
        let (ingress, egress) = *self
            .user_traffic
            .read()
            .unwrap()
            .get(&user_id)
            .unwrap_or(&(0, 0));

        match self.report_user_traffic(&user_id, ingress, egress).await {
            Ok(_) => {
                self.user_traffic.write().unwrap().remove(&user_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to report user traffic: {}", e);
                Err(e)
            }
        }
    }

    async fn report_user_traffic(
        &self,
        fingerprint: &String,
        ingress: usize,
        egress: usize,
    ) -> Result<reqwest::Response, ReportTrafficError> {
        let mut default_headers = HeaderMap::new();
        default_headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();

        let traffic_endpoint = self
            .platform_hostname()
            .join("/api/v1/metrics/traffic")
            .unwrap();

        let meter_traffic_request = MeterTrafficRequest {
            fingerprint,
            ingress,
            egress,
        };

        let request = client
            .post(traffic_endpoint.clone())
            .json(&meter_traffic_request)
            .bearer_auth(self.prepare_auth_token()?);

        request
            .send()
            .await
            .map_err(ReportTrafficError::ReqwestError)
    }

    fn prepare_auth_token(&self) -> Result<String, ReportTrafficError> {
        let service_name = self.service_name();
        let platform_name = self.platform_name();

        let mut claims = Claims::create(Duration::from_secs(60).into())
            .with_audiences(HashSet::from_strings(&[platform_name]))
            .with_subject(service_name)
            .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30).into());

        claims.create_nonce();
        claims.issued_at = Some(Clock::now_since_epoch());

        self.service_signing_key()
            .sign(claims)
            .map_err(ReportTrafficError::JWTError)
    }
}

#[derive(Serialize)]
pub struct MeterTrafficRequest<'a> {
    fingerprint: &'a String,
    ingress: usize,
    egress: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ReportTrafficError {
    #[error("failed to send request: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("failed to send request: {0}")]
    JWTError(#[from] jwt_simple::Error),
}
