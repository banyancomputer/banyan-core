use std::task::{Context, Poll};

use http::{Request, Response};
use http_body::Body;
use tokio::sync::oneshot;

use tower_service::Service;

use crate::traffic_counter::body::{
    DefaultOnResponse, OnResponse, RequestCounter, ResponseCounter,
};
use crate::traffic_counter::future::ResponseFuture;

#[derive(Clone, Debug)]
pub struct TrafficCounter<S, OnResponseT = DefaultOnResponse> {
    inner: S,
    on_response: OnResponseT,
}

impl<S> TrafficCounter<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            on_response: DefaultOnResponse::default(),
        }
    }
}

impl<ReqBody, ResBody, OnResponseT, S> Service<Request<ReqBody>> for TrafficCounter<S, OnResponseT>
where
    ResBody: Body,
    OnResponseT: OnResponse + Clone,
    S: Service<Request<RequestCounter<ReqBody>>, Response = Response<ResBody>>,
{
    type Response = Response<ResponseCounter<ResBody, OnResponseT>>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future, OnResponseT>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let (tx_bytes_received, rx_bytes_received) = oneshot::channel::<usize>();
        let req = req.map(|body| RequestCounter::new(body, tx_bytes_received));
        let request_info = (&req).into();
        let res = ResponseFuture {
            inner: self.inner.call(req),
            request_info,
            rx_bytes_received,
            on_response: Some(self.on_response.clone()),
        };
        res
    }
}
