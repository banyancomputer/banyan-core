use crate::traffic_counter::body::FnOnResponseEnd;
use tower_layer::Layer;

use crate::traffic_counter::service::TrafficCounter;

#[derive(Clone, Debug)]
pub struct TrafficCounterLayer {
    pub on_response_end: FnOnResponseEnd,
}

impl TrafficCounterLayer {
    pub fn new(on_response_end: FnOnResponseEnd) -> Self {
        Self { on_response_end }
    }
}

impl<S> Layer<S> for TrafficCounterLayer {
    type Service = TrafficCounter<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TrafficCounter::new(inner, self.on_response_end)
    }
}
