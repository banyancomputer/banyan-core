use crate::traffic_counter::body::OnResponseEnd;
use tower_layer::Layer;

use crate::traffic_counter::service::TrafficCounter;

#[derive(Clone, Debug)]
pub struct TrafficCounterLayer<T> {
    pub on_response_end: T,
}

impl<T> TrafficCounterLayer<T>
where
    T: OnResponseEnd,
{
    pub fn new(on_response_end: T) -> Self {
        Self { on_response_end }
    }
}

impl<S, T> Layer<S> for TrafficCounterLayer<T> {
    type Service = TrafficCounter<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TrafficCounter::new(inner)
    }
}
