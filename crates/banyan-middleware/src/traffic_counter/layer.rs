use crate::traffic_counter::body::OnResponse;
use tower_layer::Layer;

use crate::traffic_counter::service::TrafficCounter;

#[derive(Clone, Debug)]
pub struct TrafficCounterLayer<T> {
    on_response: T,
}

impl<T> TrafficCounterLayer<T>
where
    T: OnResponse,
{
    pub fn new(on_response: T) -> Self {
        Self { on_response }
    }
}

impl<S, T> Layer<S> for TrafficCounterLayer<T> {
    type Service = TrafficCounter<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TrafficCounter::new(inner)
    }
}
