use std::task::{Context, Poll};

use http::{Request, Response};
use http_body::Body;
use tokio::sync::oneshot;

use tower_service::Service;

use crate::traffic_counter::body::ResponseCounter;
use crate::traffic_counter::future::ResponseFuture;

#[derive(Clone, Debug)]
pub struct TrafficCounter<S> {
    inner: S,
}

impl<S> TrafficCounter<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for TrafficCounter<S>
where
    ResBody: Body,
    // TODO: Comment out this line and line 40 to reproduce non-implemented trait bound
    // S: Service<Request<RequestCounter<ReqBody>>, Response = Response<ResBody>>,
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = Response<ResponseCounter<ResBody>>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let (tx_bytes_received, rx_bytes_received) = oneshot::channel::<usize>();
        // TODO: Comment out this line and line 25 to reproduce non-implemented trait bound
        // let req = req.map(|body| RequestCounter::new(body, tx_bytes_received));
        let request_id = req
            .headers()
            .get("x-banyan-request-id")
            .map_or_else(String::new, |id| id.to_str().unwrap_or("").to_string());

        let res = ResponseFuture {
            inner: self.inner.call(req),
            rx_bytes_received,
            request_id,
        };
        res
    }
}
