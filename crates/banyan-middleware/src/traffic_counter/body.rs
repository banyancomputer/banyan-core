use std::error::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Buf;
use futures_util::ready;
use http::HeaderMap;
use http_body::{Body, SizeHint};
use pin_project_lite::pin_project;
use tokio::sync::oneshot;

pin_project! {
    #[derive(Debug)]
    pub struct RequestCounter<B> {
        total_bytes: usize,
        tx_bytes: Option<oneshot::Sender<usize>>,
        #[pin]
        inner: B,
    }
}

impl<B> RequestCounter<B> {
    pub fn new(inner: B, tx_bytes: oneshot::Sender<usize>) -> Self {
        Self {
            total_bytes: 0,
            tx_bytes: Some(tx_bytes),
            inner,
        }
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }
}

pin_project! {
    #[derive(Debug)]
    pub struct ResponseCounter<B> {
        total_bytes: usize,
        total_request_bytes: usize,
        request_id: String,
        // method: String,
        // path: String,
        // version: String,
        // response_end_callback:
        #[pin]
        inner: B,
    }
}

impl<B> ResponseCounter<B> {
    pub fn new(inner: B, request_id: String, total_request_bytes: usize) -> Self {
        Self {
            total_bytes: 0,
            total_request_bytes,
            request_id,
            inner,
        }
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }
}

impl<B> Body for RequestCounter<B>
where
    B: Body,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
{
    type Data = B::Data;
    type Error = Box<dyn Error + Send + Sync>;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();

        let res = match ready!(this.inner.poll_data(cx)) {
            None => {
                println!(
                    "RequestCounter poll_data() end of stream {}",
                    this.total_bytes
                );
                if let Some(tx_bytes) = this.tx_bytes.take() {
                    if tx_bytes.send(*this.total_bytes).is_err() {
                        println!("Failed to send total bytes - receiver might be dropped");
                    }
                }
                None
            }
            Some(Ok(data)) => {
                *this.total_bytes += data.remaining();
                println!(
                    "RequestCounter poll_data() accumulated size {:?}",
                    this.total_bytes
                );
                Some(Ok(data))
            }
            Some(Err(err)) => Some(Err(err.into())),
        };
        Poll::Ready(res)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        let this = self.project();
        println!(
            "RequestCounter poll_trailers() start size {:?}",
            this.total_bytes
        );
        let res = match this.inner.poll_trailers(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(data)) => Ok(data),
            Poll::Ready(Err(err)) => Err(err.into()),
        };

        println!(
            "RequestCounter poll_trailers() end size {:?}",
            this.total_bytes
        );
        Poll::Ready(res)
    }

    fn is_end_stream(&self) -> bool {
        println!(
            "RequestCounter is_end_stream() RequestCounter size {:?}",
            self.total_bytes
        );
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        println!(
            "RequestCounter size_hint() size {:?} size_hint {:?}",
            self.total_bytes,
            self.inner.size_hint()
        );
        self.inner.size_hint()
    }
}

impl<B> Body for ResponseCounter<B>
where
    B: Body,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
{
    type Data = B::Data;
    type Error = Box<dyn Error + Send + Sync>;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();

        let res = match ready!(this.inner.poll_data(cx)) {
            None => {
                println!(
                    "ResponseCounter poll_data() end of stream {}",
                    this.total_bytes
                );
                None
            }
            Some(Ok(data)) => {
                *this.total_bytes += data.remaining();
                println!(
                    "ResponseCounter poll_data() accumulated size {:?}",
                    this.total_bytes
                );
                Some(Ok(data))
            }
            Some(Err(err)) => Some(Err(err.into())),
        };
        Poll::Ready(res)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        let this = self.project();
        println!(
            "ResponseCounter poll_trailers() start size {:?}",
            this.total_bytes
        );
        let res = match this.inner.poll_trailers(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(data)) => Ok(data),
            Poll::Ready(Err(err)) => Err(err.into()),
        };

        println!(
            "ResponseCounter poll_trailers() end size {:?}",
            this.total_bytes
        );
        Poll::Ready(res)
    }

    fn is_end_stream(&self) -> bool {
        let end_stream = self.inner.is_end_stream();
        println!(
            "total bytes {} total request bytes {}",
            self.total_bytes, self.total_request_bytes
        );
        // dispatch callback
        if end_stream {
            // if let Some(tx) = self.tx.take() {
            //     if tx.send(this.total_bytes).is_err() {
            //         tracing::error!("Failed to send total bytes - receiver might be dropped");
            //     }
            // }
        }
        end_stream
    }

    fn size_hint(&self) -> SizeHint {
        println!(
            "ResponseCounter size_hint() size {:?} size_hint {:?}",
            self.total_bytes,
            self.inner.size_hint()
        );
        self.inner.size_hint()
    }
}

#[cfg(test)]
mod tests {
    use std::task::{Context, Poll};

    use bytes::{Buf, Bytes};
    use http_body::Full;

    use super::*;

    async fn poll_body_to_completion<B>(mut body: RequestCounter<B>) -> usize
    where
        B: Body + Unpin,
        B::Data: Buf,
        B::Error: Into<Box<dyn Error + Send + Sync>>, // Add this line
    {
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());

        while let Poll::Ready(data_opt) = Pin::new(&mut body).as_mut().poll_data(&mut cx) {
            if let None = data_opt {
                break;
            }
        }

        body.total_bytes
    }

    #[tokio::test]
    async fn counts_ingress_correctly_for_single_chunk() {
        let data = Bytes::from_static(b"Hello, world!");
        let body = Full::new(data.clone());
        let body_counter = RequestCounter::new(body);

        let counted_size = poll_body_to_completion(body_counter).await;
        assert_eq!(counted_size, data.len());
    }

    #[tokio::test]
    async fn counts_ingress_correctly_for_multiple_chunks() {
        let chunk1 = Bytes::from_static(b"Hello, ");
        let chunk2 = Bytes::from_static(b"world!");
        let body = Full::new(chunk1.chain(chunk2));
        let body_counter = RequestCounter::new(body);

        let counted_size = poll_body_to_completion(body_counter).await;
        assert_eq!(counted_size, "Hello, world!".len());
    }

    #[tokio::test]
    async fn counts_ingress_correctly_for_empty_body() {
        let body = Full::new(Bytes::new());
        let body_counter = RequestCounter::new(body);

        let counted_size = poll_body_to_completion(body_counter).await;
        assert_eq!(counted_size, 0);
    }
}
