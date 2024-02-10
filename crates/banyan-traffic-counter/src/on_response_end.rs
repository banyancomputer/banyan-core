use crate::body::{RequestInfo, ResponseInfo};

pub trait OnResponseEnd<B> {
    fn on_response_end(&self, req_info: &RequestInfo, res_info: &ResponseInfo);
}

impl<B, F> OnResponseEnd<B> for F
where
    F: Fn(&RequestInfo, &ResponseInfo),
{
    fn on_response_end(&self, request_info: &RequestInfo, response_info: &ResponseInfo) {
        self(request_info, response_info)
    }
}

pub struct DefaultOnResponseEnd {}

impl<B> OnResponseEnd<B> for DefaultOnResponseEnd {
    fn on_response_end(&self, _: &RequestInfo, _: &ResponseInfo) {}
}
