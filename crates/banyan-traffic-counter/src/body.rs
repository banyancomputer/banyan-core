use std::error::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Buf;
use futures_util::ready;
use http::{HeaderMap, HeaderName, Method, Request, StatusCode, Uri, Version};
use http_body::{Body, SizeHint};
use pin_project_lite::pin_project;
use tokio::sync::oneshot;

use crate::on_response_end::OnResponseEnd;
use crate::service::Session;

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
}

pin_project! {
    #[derive(Debug)]
    pub struct ResponseCounter<B, OnResponseEnd> {
        response_info: ResponseInfo,
        request_info: RequestInfo ,
        on_response_end: OnResponseEnd,
        #[pin]
        inner: B,
    }
}

impl<B, OnResponseEndT> ResponseCounter<B, OnResponseEndT>
where
    B: Body,
    OnResponseEndT: OnResponseEnd<B>,
{
    pub fn new(
        inner: B,
        headers: &HeaderMap,
        request_info: RequestInfo,
        status_code: StatusCode,
        on_response_end: OnResponseEndT,
        session: Session,
    ) -> Self {
        let response_header_bytes = headers
            .iter()
            .map(|(k, v)| k.as_str().len() + v.as_bytes().len() + ": ".len() + "\r\n".len())
            .sum();

        Self {
            request_info,
            response_info: ResponseInfo {
                session,
                body_bytes: 0,
                header_bytes: response_header_bytes,
                status_code,
            },
            on_response_end,
            inner,
        }
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
                        tracing::warn!("Failed to send total request bytes");
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

impl<B, OnResponseEndT> Body for ResponseCounter<B, OnResponseEndT>
where
    B: Body,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
    OnResponseEndT: OnResponseEnd<B::Data>,
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
                this.on_response_end
                    .on_response_end(this.request_info, this.response_info);
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
            self.on_response_end
                .on_response_end(&self.request_info, &self.response_info);
        }
        end_stream
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RequestInfo {
    pub request_id: Option<String>,
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
    pub session: Session,
}

impl<T> From<&Request<T>> for RequestInfo {
    fn from(req: &Request<T>) -> Self {
        let request_id = req
            .headers()
            .get(HeaderName::from_static("x-request-id"))
            .and_then(|v| v.to_str().ok())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let header_bytes = req
            .headers()
            .iter()
            .map(|(k, v)| k.as_str().len() + v.as_bytes().len() + ": ".len() + "\r\n".len())
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
mod tests {
    use std::task::{Context, Poll};

    use bytes::{Buf, Bytes};
    use http_body::Full;
    use tokio::sync::oneshot::Receiver;

    use super::*;

    async fn poll_to_completion<B>(mut body: B, rx: Option<Receiver<usize>>) -> Option<usize>
    where
        B: Body + Unpin,
        B::Data: Buf,
        B::Error: Into<Box<dyn Error + Send + Sync>>,
    {
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        while let Poll::Ready(data_opt) = Pin::new(&mut body).as_mut().poll_data(&mut cx) {
            if data_opt.is_none() {
                break;
            }
        }

        match rx {
            Some(rx) => Some(rx.await.unwrap_or(0)),
            None => None,
        }
    }

    #[tokio::test]
    async fn counts_ingress_correctly_for_single_chunk() {
        let data = Bytes::from_static(b"Hello, world!");
        let body = Full::new(data.clone());
        let (tx, rx) = oneshot::channel();
        let body_counter = RequestCounter::new(body, tx);

        let counted_size = poll_to_completion(body_counter, Some(rx)).await;
        assert!(counted_size.is_some());
        assert_eq!(counted_size.unwrap(), data.len());
    }

    #[tokio::test]
    async fn counts_ingress_correctly_for_empty_body() {
        let body = Full::new(Bytes::new());
        let (tx, rx) = oneshot::channel();
        let body_counter = RequestCounter::new(body, tx);

        let counted_size = poll_to_completion(body_counter, Some(rx)).await;
        assert!(counted_size.is_some());
        assert_eq!(counted_size.unwrap(), 0);
    }

    #[tokio::test]
    async fn counts_egress_correctly_for_single_chunk() {
        let data = Bytes::from_static(b"Goodbye, world!");
        let body = Full::new(data.clone());
        let headers = HeaderMap::new();
        let request_info = RequestInfo::default();
        let status_code = StatusCode::OK;
        let body_counter = ResponseCounter::new(
            body,
            &headers,
            request_info,
            status_code,
            |req_info: &RequestInfo, res_info: &ResponseInfo| {
                assert_eq!(req_info.body_bytes + req_info.header_bytes, 0);
                assert_eq!(res_info.body_bytes + res_info.header_bytes, 15);
            },
            Session::default(),
        );

        poll_to_completion(body_counter, None).await;
    }

    #[tokio::test]
    async fn counts_egress_correctly_for_empty_body() {
        let body = Full::new(Bytes::new());
        let headers = HeaderMap::new();
        let request_info = RequestInfo::default();
        let status_code = StatusCode::OK;
        let body_counter = ResponseCounter::new(
            body,
            &headers,
            request_info,
            status_code,
            |req_info: &RequestInfo, res_info: &ResponseInfo| {
                assert_eq!(req_info.body_bytes + req_info.header_bytes, 0);
                assert_eq!(res_info.body_bytes + res_info.header_bytes, 0);
            },
            Session::default(),
        );
        poll_to_completion(body_counter, None).await;
    }
}
