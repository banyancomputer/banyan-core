use serde::Serialize;
use tokio::sync::broadcast;

pub type EventBusReceiver = broadcast::Receiver<(SystemEvent, Vec<u8>)>;

pub type EventBusSender = broadcast::Sender<(SystemEvent, Vec<u8>)>;

#[derive(Clone)]
pub struct EventBus {
    bus: EventBusSender,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        let (bus, _) = broadcast::channel(1_024);
        Self { bus }
    }

    pub fn send(
        &self,
        event: SystemEvent,
        payload: &impl Serialize,
    ) -> Result<usize, EventBusError> {
        let bytes = serde_json::ser::to_vec(payload).map_err(EventBusError::Serialization)?;

        self.bus
            .send((event, bytes))
            .map_err(EventBusError::SendFailed)
    }

    pub fn subscribe(&self) -> EventBusReceiver {
        self.bus.subscribe()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("failed to send message to the event bus: {0}")]
    SendFailed(broadcast::error::SendError<(SystemEvent, Vec<u8>)>),

    #[error("unable to serialize event payload: {0}")]
    Serialization(serde_json::Error),
}

#[derive(Clone, Debug)]
pub enum SystemEvent {
    DeviceKeyRegistration,
}
