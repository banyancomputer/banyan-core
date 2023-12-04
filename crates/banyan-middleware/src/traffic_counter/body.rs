use std::error::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Buf;
use http_body::{Body, SizeHint};

use http::HeaderMap;
use pin_project_lite::pin_project;

pin_project! {
    /// A body wrapper for counting the ingress bytes.
    #[derive(Clone, Copy, Debug)]
    pub struct BodyCounter<B> {
        total_bytes: usize,
        #[pin]
        inner: B,
    }
}

impl<B> BodyCounter<B> {
    /// Create a new `BodyCounter`.
    pub fn new(inner: B) -> Self {
        Self {
            total_bytes: 0,
            inner,
        }
    }

    /// Returns the total ingress bytes counted so far.
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }
}

impl<B> Body for BodyCounter<B>
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
        let res = match this.inner.poll_data(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(None) => None,
            Poll::Ready(Some(Ok(data))) => {
                *this.total_bytes += data.remaining();
                Some(Ok(data))
            }
            Poll::Ready(Some(Err(err))) => Some(Err(err.into())),
        };

        Poll::Ready(res)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        let this = self.project();
        let res = match this.inner.poll_trailers(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(data)) => Ok(data),
            Poll::Ready(Err(err)) => Err(err.into()),
        };

        Poll::Ready(res)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        match u64::try_from(self.total_bytes) {
            Ok(n) => {
                let mut hint = self.inner.size_hint();
                if hint.lower() >= n {
                    hint.set_exact(n)
                } else if let Some(max) = hint.upper() {
                    hint.set_upper(n.min(max))
                } else {
                    hint.set_upper(n)
                }
                hint
            }
            Err(_) => self.inner.size_hint(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::task::{Context, Poll};

    use bytes::{Buf, Bytes};
    use http_body::Full;

    use super::*;

    async fn poll_body_to_completion<B>(mut body: BodyCounter<B>) -> usize
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
        let body_counter = BodyCounter::new(body);

        let counted_size = poll_body_to_completion(body_counter).await;
        assert_eq!(counted_size, data.len());
    }

    #[tokio::test]
    async fn counts_ingress_correctly_for_multiple_chunks() {
        let chunk1 = Bytes::from_static(b"Hello, ");
        let chunk2 = Bytes::from_static(b"world!");
        let body = Full::new(chunk1.chain(chunk2));
        let body_counter = BodyCounter::new(body);

        let counted_size = poll_body_to_completion(body_counter).await;
        assert_eq!(counted_size, "Hello, world!".len());
    }

    #[tokio::test]
    async fn counts_ingress_correctly_for_empty_body() {
        let body = Full::new(Bytes::new());
        let body_counter = BodyCounter::new(body);

        let counted_size = poll_body_to_completion(body_counter).await;
        assert_eq!(counted_size, 0);
    }
}
