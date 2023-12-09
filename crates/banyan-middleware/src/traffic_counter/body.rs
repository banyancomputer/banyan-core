use std::error::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Buf;
use futures_util::ready;
use http::{HeaderMap, HeaderName, Method, Request, Uri, Version};
use http_body::{Body, SizeHint};
use pin_project_lite::pin_project;
use tokio::sync::oneshot;

pin_project! {
    #[derive(Debug)]
    pub struct RequestCounter<B> {
        total_request_bytes: usize,
        tx_bytes: Option<oneshot::Sender<usize>>,
        #[pin]
        inner: B,
    }
}

impl<B> RequestCounter<B> {
    pub fn new(inner: B, tx_bytes: oneshot::Sender<usize>) -> Self {
        Self {
            total_request_bytes: 0,
            tx_bytes: Some(tx_bytes),
            inner,
        }
    }

    pub fn total_request_bytes(&self) -> usize {
        self.total_request_bytes
    }
}

pin_project! {
    #[derive(Debug)]
    pub struct ResponseCounter<B, C> {
        total_response_bytes: usize,
        total_request_bytes: usize,
        request_info: RequestInfo ,
        on_response: Option<C>,
        #[pin]
        inner: B,
    }
}

impl<B, C> ResponseCounter<B, C> {
    pub fn new(
        inner: B,
        on_response: C,
        request_info: RequestInfo,
        total_request_bytes: usize,
    ) -> Self {
        Self {
            total_response_bytes: 0,
            total_request_bytes,
            request_info,
            on_response: Some(on_response),
            inner,
        }
    }

    pub fn total_response_bytes(&self) -> usize {
        self.total_response_bytes
    }
}

impl<B> Body for RequestCounter<B>
where
    B: Body,
{
    type Data = B::Data;
    type Error = B::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();

        let res = match ready!(this.inner.poll_data(cx)) {
            None => {
                if let Some(tx_bytes) = this.tx_bytes.take() {
                    if tx_bytes.send(*this.total_request_bytes).is_err() {
                        tracing::error!("Failed to send total bytes");
                    }
                }
                println!(
                    "RequestCounter poll_data() end of stream size {:?}", this.total_request_bytes
                );
                None
            }
            Some(Ok(data)) => {
                *this.total_request_bytes += data.remaining();
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
            this.total_request_bytes
        );
        let res = match this.inner.poll_trailers(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(data)) => Ok(data),
            Poll::Ready(Err(err)) => Err(err.into()),
        };
        Poll::Ready(res)
    }

    // Not called on HttpBody request
    // Only called when response is StreamBody?
    fn is_end_stream(&self) -> bool {
        println!(
            "RequestCounter is_end_stream() size {:?}",
            self.total_request_bytes
        );
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        println!(
            "RequestCounter size_hint() size {:?} size_hint {:?}",
            self.total_request_bytes,
            self.inner.size_hint()
        );
        self.inner.size_hint()
    }
}

impl<B, OnResponseT> Body for ResponseCounter<B, OnResponseT>
where
    B: Body,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
    OnResponseT: OnResponse,
{
    type Data = B::Data;
    type Error = Box<dyn Error + Send + Sync>;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();

        let res = match ready!(this.inner.poll_data(cx)) {
            // Not called when response is HttpBody
            // Called when the response is StreamBody
            None => {
                println!(
                    "ResponseCounter poll_data() end of stream size {:?}",
                    this.total_response_bytes
                );
                let request_info = this.request_info.clone();
                let response_bytes = this.total_response_bytes;
                let request_bytes = this.total_request_bytes;
                tracing::info!(
                    request_bytes = %request_bytes,
                    response_bytes = %response_bytes,
                    method = %request_info.method,
                    uri = %request_info.uri,
                    version = ?request_info.version,
                    request_id = %request_info.request_id,
                    "finished processing request",
                );
                // self.on_response.take().unwrap().on_response(request_info, total_request_bytes, total_response_bytes);
                None
            }
            Some(Ok(data)) => {
                *this.total_response_bytes += data.remaining();
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
            this.total_response_bytes
        );
        let res = match this.inner.poll_trailers(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(data)) => Ok(data),
            Poll::Ready(Err(err)) => Err(err.into()),
        };
        Poll::Ready(res)
    }

    // Not called when response is StreamBody
    // Only called when response is HttpBody
    fn is_end_stream(&self) -> bool {
        let end_stream = self.inner.is_end_stream();
        if end_stream {
            println!(
                "ResponseCounter is_end_stream() size {:?}",
                self.total_response_bytes
            );
            let request_info = self.request_info.clone();
            let response_bytes = self.total_response_bytes;
            let request_bytes = self.total_request_bytes;
            tracing::info!(
                request_bytes = %request_bytes,
                response_bytes = %response_bytes,
                method = %request_info.method,
                uri = %request_info.uri,
                version = ?request_info.version,
                request_id = %request_info.request_id,
                "finished processing request",
            )
            // self.on_response.take().unwrap().on_response(request_info, total_request_bytes, total_response_bytes);
        }
        end_stream
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

#[derive(Debug, Clone)]
pub struct RequestInfo {
    request_id: String,
    method: Method,
    uri: Uri,
    version: Version,
}

impl<T> From<&Request<T>> for RequestInfo {
    fn from(req: &Request<T>) -> Self {
        let request_id = req
            .headers()
            // x-request-id is crate private in tower-http
            .get(HeaderName::from_static("x-request-id"))
            .map_or_else(String::new, |id| id.to_str().unwrap_or("").to_string());

        RequestInfo {
            request_id,
            method: req.method().clone(),
            uri: req.uri().clone(),
            version: req.version(),
        }
    }
}

pub trait OnResponse {
    fn on_response(self, req_info: RequestInfo, read_bytes: usize, write_bytes: usize);
}

#[derive(Clone, Debug)]
pub struct DefaultOnResponse {}

impl Default for DefaultOnResponse {
    fn default() -> Self {
        Self {}
    }
}

impl OnResponse for DefaultOnResponse {
    fn on_response(self, req_info: RequestInfo, read_bytes: usize, write_bytes: usize) {}
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

        body.total_request_bytes
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
