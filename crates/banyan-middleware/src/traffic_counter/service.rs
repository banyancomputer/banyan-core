use std::task::{Context, Poll};

use http_body::Body;
use tower::Service;

use http::{Request, Response};

use crate::traffic_counter::body::BodyCounter;
use crate::traffic_counter::future::ResponseFuture;
use crate::traffic_counter::X_BANYAN_REQUEST_SIZE;

// TrafficCounter Middleware
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
    S: Service<Request<BodyCounter<ReqBody>>, Response = Response<ResBody>>,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut req = req.map(|body| BodyCounter::new(body));
        let request_size = req.body().total_bytes();

        req.headers_mut()
            .insert(X_BANYAN_REQUEST_SIZE, request_size.into());

        ResponseFuture {
            inner: self.inner.call(req),
        }
    }
}
