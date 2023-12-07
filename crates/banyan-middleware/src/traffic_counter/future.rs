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

use crate::traffic_counter::body::ResponseCounter;

pin_project! {
    #[derive(Debug)]
    pub struct ResponseFuture<F> {
        #[pin]
        pub(crate) inner: F,
        pub rx_bytes_received: oneshot::Receiver<usize>,
        pub request_id: String
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
        println!("ResponseFuture poll() start size");
        // return this.inner.poll(cx);
        // begin processing response
        let result = ready!(this.inner.poll(cx));

        // If the receiver is set, try polling it
        // let bytes= ready!(this.rx.poll(cx));
        let total_request_bytes = match this.rx_bytes_received.try_recv() {
            Ok(bytes_received) => {
                println!("ResponseFuture poll() end of stream {}", bytes_received);
                bytes_received
            }
            Err(err) => {
                println!("ResponseFuture poll() no bytes {:?}", err);
                0
            }
        };

        // await for one shot channel
        // this.rx.poll_unpin(cx);
        match result {
            Ok(res) => {
                let (parts, body) = res.into_parts();
                let res = Response::from_parts(
                    parts,
                    ResponseCounter::new(body, this.request_id.clone(), total_request_bytes),
                );
                println!("ResponseFuture poll() end size");
                Poll::Ready(Ok(res))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
