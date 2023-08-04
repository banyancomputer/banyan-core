use std::fmt::{self, Display, Formatter};
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::{async_trait, RequestExt};
use axum::body::{Body, Bytes, HttpBody};
use axum::extract::{BodyStream, FromRequest};
use axum::http::{Request, StatusCode};
use axum::http::header::{HeaderMap, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use futures::stream::Stream;

#[derive(Debug)]
pub struct JsonMultipartUpload<T> {
    pub json: T,
    pub car_stream: BodyStream,
}

#[async_trait]
impl<T, S, B> FromRequest<T, S, B> for JsonMultipartUpload<T>
where
    B: HttpBody + Send + 'static,
    S: Send + Sync,
    T: serde::de::DeserializeOwned,
{
    type Rejection = ();

    async fn from_request(req: Request<B>, _state: &S) -> Result<Self, Self::Rejection> {
        let boundary = parse_boundary(req.headers()).ok_or(())?;
        let (parts, stream) = req.into_parts();

        let constraints = multer::Constraints::new()
            .allowed_fields(vec!["request-data", "car-upload"])
            .size_limit(multer::SizeLimit::new()
                .for_field("request-data", REQUEST_DATA_SIZE_LIMIT)
                .for_field("car-upload", CAR_DATA_SIZE_LIMIT)
            );

        let multipart = multer::Multipart::with_constraints(stream, boundary, constraints);

        let car_stream_field = multipart.next_field().await.unwrap();
        // validate name is car-upload (request_data_field.name())
        // validate type is "application/vnd.ipld.car; version=2" (request_data_field.content_type())
        let car_stream = BodyStream::from_request(Request::from_parts(parts.clone(), car_stream_field));

        Ok(Self { data, car_stream })
    }
}
