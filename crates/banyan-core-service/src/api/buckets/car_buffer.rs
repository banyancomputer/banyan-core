use bytes::Bytes;

// We only look through the first MiB for the keys section. It's considered badly formed if not
// found in that window.
const SCANNED_CAR_CAPACITY: usize = 1_048_576;

enum BufferState {
    Buffering(Vec<u8>),
    Done,
    Empty,
}

pub struct CarBuffer {
    state: BufferState,
}

impl CarBuffer {
    pub fn add_chunk(&mut self, bytes: &Bytes) {
        // Once we've received all the data we want, we want this to do as little as possible
        if let BufferState::Done = self.state {
            return;
        }

        self.state = match std::mem::replace(&mut self.state, BufferState::Empty) {
            // This is the first chunk
            BufferState::Empty => {
                let data = bytes.clone().into_iter().take(SCANNED_CAR_CAPACITY).collect();
                BufferState::Buffering(data)
            }
            // We've already some data, we just need to extend it to cover as much as possible
            BufferState::Buffering(mut vec) => {
                let remaining_space = SCANNED_CAR_CAPACITY - vec.len();
                let consumable_bytes = remaining_space.min(bytes.len());

                // Add as many bytes as we can to the buffer
                vec.extend(bytes.slice(0..consumable_bytes));

                BufferState::Buffering(vec)
            },
            _ => panic!("we already checked for done"),
        };
    }

    fn as_slice(&self) -> &[u8] {
        match &self.state {
            BufferState::Buffering(vec) => vec.as_slice(),
            _ => &[],
        }
    }

    fn is_done(&self) -> bool {
        match &self.state {
            BufferState::Done => true,
            _ => false,
        }
    }

    pub fn is_ready(&self) -> bool {
        match &self.state {
            BufferState::Buffering(_) => true,
            _ => false,
        }
    }

    pub fn new() -> Self {
        Self {
            state: BufferState::Empty,
        }
    }

    pub fn parse(&mut self) -> Result<Option<()>, &str> {
        if self.is_done() {
            return Err("already finished");
        }

        if !self.is_ready() {
            return Ok(None);
        }

        // todo: process data buffer
        let _ = self.as_slice();
        self.state = BufferState::Done;

        // todo: return something useful
        Ok(Some(()))
    }
}
