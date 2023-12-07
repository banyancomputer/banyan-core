use crate::traffic_counter::service::TrafficCounter;
use tower_layer::Layer;

#[derive(Clone, Debug)]
pub struct TrafficCounterLayer;

impl TrafficCounterLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S> Layer<S> for TrafficCounterLayer {
    type Service = TrafficCounter<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TrafficCounter::new(inner)
    }
}
