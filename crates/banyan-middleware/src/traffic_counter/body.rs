use std::error::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Buf;
use futures_util::ready;
use http::{HeaderMap, HeaderName, Method, Request, StatusCode, Uri, Version};
use http_body::{Body, SizeHint};
use pin_project_lite::pin_project;
use tokio::sync::oneshot;

pin_project! {
    #[derive(Debug)]
    pub struct RequestCounter<B> {
        bytes_from_stream: usize,
        tx_bytes: Option<oneshot::Sender<usize>>,
        #[pin]
        inner: B,
    }
}

impl<B> RequestCounter<B> {
    pub fn new(inner: B, tx_bytes: oneshot::Sender<usize>) -> Self {
        Self {
            bytes_from_stream: 0,
            tx_bytes: Some(tx_bytes),
            inner,
        }
    }

    pub fn bytes_from_stream(&self) -> usize {
        self.bytes_from_stream
    }
}
pub type FnOnResponseEnd = fn(req_info: &RequestInfo, res_info: &ResponseInfo) -> ();

pin_project! {
    #[derive(Debug)]
    pub struct ResponseCounter<B> {
        response_info: ResponseInfo,
        request_info: RequestInfo ,
        on_response_end: Option<FnOnResponseEnd>,
        #[pin]
        inner: B,
    }
}

impl<B> ResponseCounter<B> {
    pub fn new(
        inner: B,
        headers: &HeaderMap,
        request_info: RequestInfo,
        status_code: StatusCode,
        on_response_end: Option<FnOnResponseEnd>,
    ) -> Self {
        let response_header_bytes = headers
            .iter()
            .map(|(k, v)| k.as_str().len() + v.as_bytes().len())
            .sum();

        Self {
            request_info,
            response_info: ResponseInfo {
                body_bytes: 0,
                header_bytes: response_header_bytes,
                status_code,
            },
            on_response_end,
            inner,
        }
    }

    pub fn total_response_bytes(&self) -> usize {
        self.response_info.header_bytes + self.response_info.body_bytes
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
        match ready!(this.inner.poll_data(cx)) {
            Some(Ok(data)) => {
                *this.bytes_from_stream += data.chunk().len();
                Poll::Ready(Some(Ok(data)))
            }
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            // Will not get called at all if request body is empty
            None => {
                if let Some(tx_bytes) = this.tx_bytes.take() {
                    if tx_bytes.send(*this.bytes_from_stream).is_err() {
                        tracing::error!("Failed to send total request bytes");
                    }
                }
                Poll::Ready(None)
            }
        }
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
                        *this.bytes_from_stream += name.as_str().len() + value.as_bytes().len();
                    }
                }
                Ok(data)
            }
            Poll::Ready(Err(err)) => Err(err),
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
            Some(Ok(data)) => {
                this.response_info.body_bytes += data.chunk().len();
                Some(Ok(data))
            }
            Some(Err(err)) => Some(Err(err.into())),
            // Not called when response is HttpBody
            // Called when the response is StreamBody
            None => {
                if let Some(on_response_end) = &this.on_response_end {
                    on_response_end(this.request_info, this.response_info);
                }
                None
            }
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
                        this.response_info.header_bytes +=
                            name.as_str().len() + value.as_bytes().len();
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
            if let Some(on_response_end) = &self.on_response_end {
                on_response_end(&self.request_info, &self.response_info);
            }
        }
        end_stream
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RequestInfo {
    pub request_id: String,
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub header_bytes: usize,
    pub body_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct ResponseInfo {
    pub body_bytes: usize,
    pub header_bytes: usize,
    pub status_code: StatusCode,
}

impl<T> From<&Request<T>> for RequestInfo {
    fn from(req: &Request<T>) -> Self {
        let request_id = req
            .headers()
            // x-request-id is crate private in tower-http
            .get(HeaderName::from_static("x-request-id"))
            .map_or_else(String::new, |id| {
                id.to_str().unwrap_or_default().to_string()
            });
        let header_bytes = req
            .headers()
            .iter()
            .map(|(k, v)| k.as_str().len() + v.as_bytes().len())
            .sum();

        RequestInfo {
            request_id,
            method: req.method().clone(),
            uri: req.uri().clone(),
            version: req.version(),
            header_bytes,
            body_bytes: 0,
        }
    }
}

#[cfg(test)]
mod tests {}
