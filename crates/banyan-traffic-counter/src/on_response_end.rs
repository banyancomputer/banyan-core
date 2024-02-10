use crate::body::{RequestInfo, ResponseInfo};

pub trait OnResponseEnd<B> {
    fn on_response_end(&self, req_info: &RequestInfo, res_info: &ResponseInfo);
}

impl<B> OnResponseEnd<B> for () {
    #[inline]
    fn on_response_end(&self, _: &RequestInfo, _: &ResponseInfo) {}
}

impl<B, F> OnResponseEnd<B> for F
where
    F: Fn(&RequestInfo, &ResponseInfo),
{
    fn on_response_end(&self, request_info: &RequestInfo, response_info: &ResponseInfo) {
        self(request_info, response_info)
    }
}

#[derive(Clone, Debug)]
pub struct DefaultOnResponseEnd {}

impl Default for DefaultOnResponseEnd {
    fn default() -> Self {
        Self {}
    }
}

impl DefaultOnResponseEnd {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<B> OnResponseEnd<B> for DefaultOnResponseEnd {
    fn on_response_end(&self, _: &RequestInfo, _: &ResponseInfo) {}
}
