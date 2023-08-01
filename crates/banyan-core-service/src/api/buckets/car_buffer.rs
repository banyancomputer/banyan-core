use bytes::Bytes;

// We only look through the first MiB for the keys section. It's considered badly formed if not
// found in that window.
const SCANNED_CAR_CAPACITY: usize = 1_048_576;

enum BufferState {
    Data(Vec<u8>),
    Empty,
}

pub struct CarBuffer {
    state: BufferState,
}

impl CarBuffer {
    pub fn add_chunk(&mut self, bytes: &Bytes) {
        // Once we've received all the data we want, we want this to do as little as possible
        if self.done() {
            return;
        }

        self.state = match std::mem::replace(&mut self.state, BufferState::Empty) {
            // This is the first chunk
            BufferState::Empty => {
                let data = bytes.clone().into_iter().take(SCANNED_CAR_CAPACITY).collect();
                BufferState::Data(data)
            }
            // We've already some data, we just need to extend it to cover as much as possible
            BufferState::Data(mut vec) => {
                let remaining_space = SCANNED_CAR_CAPACITY - vec.len();
                let consumable_bytes = remaining_space.min(bytes.len());

                // Add as many bytes as we can to the buffer
                vec.extend(bytes.slice(0..consumable_bytes));

                BufferState::Data(vec)
            },
        };
    }

    fn as_slice(&self) -> &[u8] {
        match &self.state {
            BufferState::Data(vec) => vec.as_slice(),
            BufferState::Empty => &[],
        }
    }

    fn done(&self) -> bool {
        match &self.state {
            BufferState::Data(vec) => vec.len() >= SCANNED_CAR_CAPACITY,
            BufferState::Empty => false,
        }
    }

    pub fn new() -> Self {
        Self {
            state: BufferState::Empty,
        }
    }

    pub fn parse(&self) -> Result<Option<()>, &str> {
        // todo: should be some kind of ready state or something, might want to separate a ready() method
        // and a parse() method and keep done() a separate thing.
        if !self.done() {
            return Ok(None);
        }

        // todo: process data buffer
        let _ = self.as_slice();

        // todo: return something useful
        Ok(Some(()))
    }
}
