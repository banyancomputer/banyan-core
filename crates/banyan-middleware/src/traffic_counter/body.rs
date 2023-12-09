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
        on_response_end: C,
        #[pin]
        inner: B,
    }
}

impl<B, C> ResponseCounter<B, C> {
    pub fn new(
        inner: B,
        on_response_end: C,
        request_info: RequestInfo,
        total_request_bytes: usize,
    ) -> Self {
        Self {
            total_response_bytes: 0,
            total_request_bytes,
            request_info,
            on_response_end,
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
                        tracing::error!("failed to send total requst bytes");
                    }
                }
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
        let res = match this.inner.poll_trailers(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(data)) => {
                if let Some(headers) = &data {
                    for (name, value) in headers.iter() {
                        *this.total_request_bytes += name.as_str().len() + value.as_bytes().len();
                    }
                }
                Ok(data)
            }
            Poll::Ready(Err(err)) => Err(err.into()),
        };
        Poll::Ready(res)
    }

    // Not called on HttpBody request
    // Only called when response is StreamBody?
    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

impl<B, OnResponseT> Body for ResponseCounter<B, OnResponseT>
where
    B: Body,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
    OnResponseT: OnResponseEnd,
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
                this.on_response_end.on_response_end(
                    &this.request_info,
                    *this.total_request_bytes,
                    *this.total_response_bytes,
                );
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
        let res = match this.inner.poll_trailers(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(data)) => {
                if let Some(headers) = &data {
                    for (name, value) in headers.iter() {
                        *this.total_request_bytes += name.as_str().len() + value.as_bytes().len();
                    }
                }
                Ok(data)
            }
            Poll::Ready(Err(err)) => Err(err.into()),
        };
        Poll::Ready(res)
    }

    // Not called when response is StreamBody
    // Only called when response is HttpBody
    fn is_end_stream(&self) -> bool {
        let end_stream = self.inner.is_end_stream();
        if end_stream {
            self.on_response_end.on_response_end(
                &self.request_info,
                self.total_request_bytes,
                self.total_response_bytes,
            );
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

pub trait OnResponseEnd {
    fn on_response_end(&self, req_info: &RequestInfo, read_bytes: usize, write_bytes: usize);
}

#[derive(Clone, Debug)]
pub struct DefaultOnResponseEnd {}

impl Default for DefaultOnResponseEnd {
    fn default() -> Self {
        Self {}
    }
}

impl OnResponseEnd for DefaultOnResponseEnd {
    fn on_response_end(
        &self,
        request_info: &RequestInfo,
        request_bytes: usize,
        response_bytes: usize,
    ) {
        tracing::info!(
            request_bytes = %request_bytes,
            response_bytes = %response_bytes,
            method = %request_info.method,
            uri = %request_info.uri,
            version = ?request_info.version,
            request_id = %request_info.request_id,
            "finished processing request",
        );
    }
}

#[cfg(test)]
mod tests {}
