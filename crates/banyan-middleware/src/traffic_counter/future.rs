use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::ready;
use http::Response;
use http_body::Body;
use pin_project_lite::pin_project;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;

use crate::traffic_counter::body::{OnResponseEnd, RequestInfo, ResponseCounter};

pin_project! {
    #[derive(Debug)]
    pub struct ResponseFuture<F,OnResponseEnd> {
        #[pin]
        pub(crate) inner: F,
        pub rx_bytes_received: oneshot::Receiver<usize>,
        pub request_info: RequestInfo,
        pub on_response_end: Option<OnResponseEnd>
    }
}

impl<F, B, E, OnResponseT> Future for ResponseFuture<F, OnResponseT>
where
    F: Future<Output = Result<Response<B>, E>>,
    OnResponseT: OnResponseEnd,
    B: Body,
{
    type Output = Result<Response<ResponseCounter<B, OnResponseT>>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let result = ready!(this.inner.poll(cx));
        let total_request_bytes = match this.rx_bytes_received.try_recv() {
            Ok(bytes_received) => bytes_received,
            // that should not happen, since the request future would've already been dropped
            Err(TryRecvError::Empty) => {
                tracing::error!("ResponseFuture poll() end size: oneshot channel empty");
                0
            }
            // that's expected when there are no request bytes
            Err(TryRecvError::Closed) => 0,
        };

        match result {
            Ok(res) => {
                let (parts, body) = res.into_parts();
                let res = Response::from_parts(
                    parts,
                    ResponseCounter::new(
                        body,
                        this.on_response_end.take().unwrap(),
                        this.request_info.clone(),
                        total_request_bytes,
                    ),
                );
                Poll::Ready(Ok(res))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
