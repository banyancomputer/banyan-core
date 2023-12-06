use std::task::{Context, Poll};

use http::{Request, Response};
use http_body::Body;
use tower::Service;

use crate::traffic_counter::body::BodyCounter;
use crate::traffic_counter::future::ResponseFuture;

use super::X_BANYAN_REQUEST_SIZE;

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
    type Response = Response<BodyCounter<ResBody>>;
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

#[cfg(test)]
mod tests {
    // #[test]
    // fn test_service() {
    //     let mut service = TrafficCounter::new(|req: Request<BodyCounter<Full<Bytes>>>| {
    //         let response: Response<BodyCounter<Full<Bytes>>> = Response::builder()
    //             .status(StatusCode::OK)
    //             .body(BodyCounter::new(Full::from(Bytes::from("Hello, world!"))))
    //             .unwrap();
    //         futures::future::ready(Ok::<_, ()>(response))
    //     });

    //     let request: Request<Full<Bytes>> = Request::builder()
    //         .body(Full::from(Bytes::from("Hello, world!")))
    //         .unwrap();

    //     let future = service.call(request);
    //     let result = block_on(future);

    //     assert!(result.is_ok());
    //     let response = result.unwrap();
    //     assert_eq!(response.status(), StatusCode::OK);
    //     assert_eq!(response.body().total_bytes(), 13);
    // }
}
