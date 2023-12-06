use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::ready;
use http::Response;
use http_body::Body;
use pin_project_lite::pin_project;

use crate::traffic_counter::body::BodyCounter;
use crate::traffic_counter::X_BANYAN_RESPONSE_SIZE;

pin_project! {
    #[derive(Debug)]
    pub struct ResponseFuture<F> {
        #[pin]
        pub(crate) inner: F,
    }
}

impl<F, B, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
    B: Body,
{
    type Output = Result<Response<BodyCounter<B>>, E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let res = ready!(self.as_mut().project().inner.poll(cx)?);
        let (parts, body) = res.into_parts();

        let mut res = Response::from_parts(parts, BodyCounter::new(body));

        let total_bytes = res.body().total_bytes();
        res.headers_mut()
            .insert(X_BANYAN_RESPONSE_SIZE, total_bytes.into());

        Poll::Ready(Ok(res))
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use futures::executor::block_on;
    use http::{Response, StatusCode};
    use http_body::Full;

    use super::*;

    #[test]
    fn test_response_future() {
        let response: Response<Full<Bytes>> = Response::builder()
            .status(StatusCode::OK)
            .body(Full::from(Bytes::from("Hello, world!")))
            .unwrap();

        let future = ResponseFuture {
            inner: futures::future::ready(Ok::<_, ()>(response)),
        };
        let result = block_on(future);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.body().total_bytes(), 13);
    }
}
