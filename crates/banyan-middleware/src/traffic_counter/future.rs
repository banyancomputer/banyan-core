use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use http_body::Body;

use crate::traffic_counter::body::BodyCounter;
use crate::traffic_counter::service::TrafficCounter;
use crate::traffic_counter::X_BANYAN_RESPONSE_SIZE;
use futures_util::ready;
use http::{header, Response};
use pin_project_lite::pin_project;

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
    type Output = Result<Response<B>, E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let res = ready!(self.as_mut().project().inner.poll(cx)?);
        let (mut parts, body) = res.into_parts();

        let mut res = Response::from_parts(parts, body);


        // res.headers_mut()
        //     .insert(X_BANYAN_RESPONSE_SIZE, res.body().total_bytes().into());

        Poll::Ready(Ok(res))
    }
}
