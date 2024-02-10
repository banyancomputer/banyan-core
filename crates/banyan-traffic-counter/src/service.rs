use std::task::{Context, Poll};

use http::{Request, Response};
use http_body::Body;
use tokio::sync::oneshot;
use tower_service::Service;

use crate::body::{RequestCounter, ResponseCounter};
use crate::future::ResponseFuture;
use crate::on_response_end::{DefaultOnResponseEnd, OnResponseEnd};

#[derive(Clone, Debug)]
pub struct TrafficCounter<S, OnResponseEnd = DefaultOnResponseEnd> {
    inner: S,
    on_response_end: OnResponseEnd,
}

impl<S, OnResponseEnd> TrafficCounter<S, OnResponseEnd> {
    pub fn new(inner: S, on_response_end: OnResponseEnd) -> Self {
        Self {
            inner,
            on_response_end,
        }
    }
}

impl<ReqBody, ResBody, OnResponseEndT, S> Service<Request<ReqBody>>
    for TrafficCounter<S, OnResponseEndT>
where
    ResBody: Body,
    S: Service<Request<RequestCounter<ReqBody>>, Response = Response<ResBody>>,
    OnResponseEndT: OnResponseEnd<ResBody> + Clone,
{
    type Response = Response<ResponseCounter<ResBody, OnResponseEndT>>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future, OnResponseEndT>;

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
            on_response_end: Some(self.on_response_end.clone()),
        }
    }
}
