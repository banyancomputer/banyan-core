use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use banyan_traffic_counter::body::{RequestInfo, ResponseInfo};
use banyan_traffic_counter::on_response_end::OnResponseEnd;
use http::{HeaderMap, HeaderValue};
use jwt_simple::algorithms::ECDSAP384KeyPairLike;
use jwt_simple::claims::Claims;
use jwt_simple::prelude::*;
use reqwest::Client;
use url::Url;

use crate::app::AppState;
use crate::utils::SigningKey;

#[derive(Clone)]
pub struct TrafficReporter {
    app: AppState,

    last_flush: Instant,
    user_traffic: RefCell<HashMap<String, (usize, usize)>>,
}

impl<B> OnResponseEnd<B> for TrafficReporter {
    fn on_response_end(&self, req_info: &RequestInfo, res_info: &ResponseInfo) {
        let user_id = match req_info.user_id.clone() {
            Some(id) => id,
            None => {
                tracing::error!("No user_id found in request, skipping traffic reporting");
                return;
            }
        };

        let ingress = req_info.header_bytes + req_info.body_bytes;
        let egress = res_info.header_bytes + res_info.body_bytes;
        self.log_traffic(user_id, ingress, egress);
    }
}
impl TrafficReporter {
    pub fn new(app: AppState) -> Self {
        Self {
            app,
            last_flush: Instant::now(),
            user_traffic: RefCell::new(HashMap::new()),
        }
    }

    fn platform_hostname(&self) -> Url {
        self.app.platform_hostname()
    }
    fn service_name(&self) -> &str {
        &self.app.service_name()
    }
    fn platform_name(&self) -> &str {
        &self.app.platform_name()
    }
    fn service_signing_key(&self) -> SigningKey {
        self.app.secrets().service_signing_key()
    }

    fn log_traffic(&self, user_id: String, ingress: usize, egress: usize) {
        let mut user_traffic = self.user_traffic.borrow_mut();
        let (current_ingress, current_egress) =
            user_traffic.get(&user_id).unwrap_or(&(0, 0)).clone();
        user_traffic.insert(
            user_id,
            (current_ingress + ingress, current_egress + egress),
        );
    }

    async fn flush_traffic_metrics(&mut self, user_id: String) {
        let now = Instant::now();
        let duration = now.duration_since(self.last_flush);
        if duration.as_secs() >= 60 * 60 {
            let (ingress, egress) = self
                .user_traffic
                .borrow()
                .get(&user_id)
                .unwrap_or(&(0, 0))
                .clone();

            match self.report_user_traffic(&user_id, ingress, egress).await {
                Ok(_) => {
                    self.user_traffic.borrow_mut().remove(&user_id);
                    self.last_flush = now;
                }
                Err(e) => {
                    tracing::error!("Failed to report user traffic: {}", e);
                }
            };
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
            .join("/api/metrics/traffic")
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
