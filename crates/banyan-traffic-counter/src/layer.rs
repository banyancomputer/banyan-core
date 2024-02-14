use tower_layer::Layer;

use crate::on_response_end::DefaultOnResponseEnd;
use crate::service::TrafficCounter;

#[derive(Clone, Debug)]
pub struct TrafficCounterLayer<OnResponseEnd = DefaultOnResponseEnd> {
    pub on_response_end: OnResponseEnd,
}

impl<OnResponseEnd> TrafficCounterLayer<OnResponseEnd> {
    pub fn new(on_response_end: OnResponseEnd) -> Self {
        Self { on_response_end }
    }
}

impl<S, OnResponseEnd> Layer<S> for TrafficCounterLayer<OnResponseEnd>
where
    OnResponseEnd: Clone,
{
    type Service = TrafficCounter<S, OnResponseEnd>;

    fn layer(&self, inner: S) -> Self::Service {
        TrafficCounter::new(inner, self.on_response_end.clone())
    }
}
