use std::task::{Context, Poll};

use http::{Request, Response};
use http_body::Body;
use tokio::sync::oneshot;

use tower_service::Service;

use crate::traffic_counter::body::{
    DefaultOnResponseEnd, OnResponseEnd, RequestCounter, ResponseCounter,
};
use crate::traffic_counter::future::ResponseFuture;

#[derive(Clone, Debug)]
pub struct TrafficCounter<S, OnResponseEndT = DefaultOnResponseEnd> {
    inner: S,
    on_response_end: OnResponseEndT,
}

impl<S> TrafficCounter<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            on_response_end: DefaultOnResponseEnd,
        }
    }
}

impl<ReqBody, ResBody, OnResponseEndT, S> Service<Request<ReqBody>>
    for TrafficCounter<S, OnResponseEndT>
where
    ResBody: Body,
    OnResponseEndT: OnResponseEnd + Clone,
    S: Service<Request<RequestCounter<ReqBody>>, Response = Response<ResBody>>,
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
        let (parts, body) = req.into_parts();
        let body = RequestCounter::new(body, tx_bytes_received);
        let req = Request::from_parts(parts, body);
        let request_info = (&req).into();
        ResponseFuture {
            inner: self.inner.call(req),
            request_info,
            rx_bytes_received,
            on_response_end: Some(self.on_response_end.clone()),
        }
    }
}
