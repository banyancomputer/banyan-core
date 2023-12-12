use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::ready;
use http::Response;
use http_body::Body;
use pin_project_lite::pin_project;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;

use crate::traffic_counter::body::{FnOnResponseEnd, RequestInfo, ResponseCounter};

pin_project! {
    #[derive(Debug)]
    pub struct ResponseFuture<F> {
        #[pin]
        pub(crate) inner: F,
        pub rx_bytes_received: oneshot::Receiver<usize>,
        pub request_info: RequestInfo,
        pub on_response_end: FnOnResponseEnd
    }
}

impl<F, B, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
    B: Body,
{
    type Output = Result<Response<ResponseCounter<B>>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let result = ready!(this.inner.poll(cx));
        let request_body_bytes = match this.rx_bytes_received.try_recv() {
            Ok(bytes_received) => bytes_received,
            // that should not happen, since the request future would've already been dropped
            Err(TryRecvError::Empty) => {
                tracing::error!("ResponseFuture poll() end size: oneshot channel empty");
                0
            }
            // that's expected when there are no request bytes
            Err(TryRecvError::Closed) => 0,
        };
        this.request_info.body_bytes = request_body_bytes;
        let request_info = std::mem::take(this.request_info);

        match result {
            Ok(res) => {
                let (parts, body) = res.into_parts();
                let body = ResponseCounter::new(
                    body,
                    &parts.headers,
                    request_info,
                    parts.status,
                    *this.on_response_end,
                );
                let res = Response::from_parts(parts, body);
                Poll::Ready(Ok(res))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
