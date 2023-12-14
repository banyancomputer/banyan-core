use std::task::{Context, Poll};

use http::{Request, Response};
use http_body::Body;
use tokio::sync::oneshot;
use tower_service::Service;

use crate::body::{FnOnResponseEnd, RequestCounter, ResponseCounter};
use crate::future::ResponseFuture;

#[derive(Clone, Debug)]
pub struct TrafficCounter<S> {
    inner: S,
    on_response_end: FnOnResponseEnd,
}

impl<S> TrafficCounter<S> {
    pub fn new(inner: S, on_response_end: FnOnResponseEnd) -> Self {
        Self {
            inner,
            on_response_end,
        }
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for TrafficCounter<S>
where
    ResBody: Body,
    S: Service<Request<RequestCounter<ReqBody>>, Response = Response<ResBody>>,
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
        let req = req.map(|body| RequestCounter::new(body, tx_bytes_received));
        let request_info = (&req).into();
        let inner = self.inner.call(req);
        ResponseFuture {
            inner,
            request_info,
            rx_bytes_received,
            on_response_end: self.on_response_end,
        }
    }
}
