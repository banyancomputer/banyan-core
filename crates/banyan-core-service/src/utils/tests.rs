use axum::response::Response;

pub async fn deserialize_response<T: for<'de> serde::Deserialize<'de>>(res: Response) -> T {
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn deserialize_result<T: for<'de> serde::Deserialize<'de>, E: std::fmt::Debug>(
    res: Result<Response, E>,
) -> T {
    let res = res.unwrap();
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}
