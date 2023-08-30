use std::collections::HashMap;

use blake3::Hasher;
use bytes::{Bytes, BytesMut};

const CAR_HEADER_UPPER_LIMIT: usize = 16 * 1024 * 1024; // Limit car headers to 16MiB

const CAR_FILE_UPPER_LIMIT: usize = 32 * 1024 * 1024 * 1024; // We limit individual CAR files to 32GiB

pub struct BlockMeta {
    cid: String,
    offset: usize,
    length: usize,
}

#[derive(Clone, Debug)]
enum CarState {
    Init,
    Header(usize),

    BlockStart,
    BlockSeekUntil(usize),

    Indexes,
    Complete,
}

pub struct CarReport {
    integrity_hash: String,
    total_size: usize,
}

impl CarReport {
    pub fn integrity_hash(&self) -> &str {
        self.integrity_hash.as_str()
    }

    pub fn total_size(&self) -> usize {
        self.total_size
    }
}

pub struct StreamingCarAnalyzer {
    buffer: BytesMut,
    state: CarState,
    stream_offset: usize,

    hasher: blake3::Hasher,

    expecting_index: bool,
    // todo: switch key type to fixed byte string...
    known_roots: HashMap<String, bool>,
}

impl StreamingCarAnalyzer {
    pub fn add_chunk(&mut self, bytes: &Bytes) -> Result<(), StreamingCarAnalyzerError> {
        self.exceeds_buffer_limit(bytes.len())?;

        match &self.state {
            CarState::BlockSeekUntil(desired_offset) => {
                if (self.stream_offset + bytes.len()) < *desired_offset {
                    self.stream_offset += bytes.len();
                    return Ok(());
                }

                // This new data brings us up to our desired offset, copy in the relevant bytes we're
                // looking for and transition our state to the next desired one. We are generally
                // taking more data here than we'll probably need in the parser but we'd need a more
                // complex handler to reduce that minor overhead.
                let ignored_bytes = desired_offset - self.stream_offset;

                self.buffer.extend_from_slice(&bytes[ignored_bytes..]);
                self.stream_offset += ignored_bytes;

                self.state = CarState::BlockStart;
            }
            _ => {
                // For normal states we don't have to do anything other than copy the data
                self.buffer.extend_from_slice(&bytes);
            }
        }

        Ok(())
    }

    fn exceeds_buffer_limit(&self, new_bytes: usize) -> Result<(), StreamingCarAnalyzerError> {
        let new_byte_total = self.stream_offset + new_bytes;

        if new_byte_total > CAR_FILE_UPPER_LIMIT {
            return Err(StreamingCarAnalyzerError::MaxCarSizeExceeded(new_byte_total));
        }

        Ok(())
    }

    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
            state: CarState::Init,
            stream_offset: 0,

            hasher: blake3::Hasher::new(),

            expecting_index: false,
            known_roots: HashMap::new(),
        }
    }

    pub async fn next(&mut self) -> Result<Option<BlockMeta>, StreamingCarAnalyzerError> {
        match self.state {
            CarState::Init => {
                // read varint to indicate the length of the header, transition to Header(length)
                // state, check if size exceeds our threshold

                Ok(None)
            }
            CarState::Header(header_size) => {
                if header_size >= CAR_HEADER_UPPER_LIMIT {
                    return Err(StreamingCarAnalyzerError::HeaderSegmentSizeExceeded(header_size));
                }

                if self.buffer.len() < header_size {
                    return Ok(None);
                }

                // todo parse the header
                self.stream_offset += header_size;

                // todo: need to set this to the beginning of the data blocks to skip over any
                // padding, I'll get this from parsing the header...
                self.state = CarState::BlockSeekUntil(self.stream_offset);

                Ok(None)
            }
            CarState::BlockStart => {
                Ok(None)
            }

            // Waiting on more data, can't do anything yet, may need to check this anyway if I end
            // up looping here to transition between multiple states in one next() call...
            CarState::BlockSeekUntil(_) => Ok(None),

            // Placeholder for all our states, get rid of
            _ => Ok(None),
        }
    }

    pub fn report(self) -> Result<CarReport, StreamingCarAnalyzerError> {
        Ok(CarReport {
            integrity_hash: self.hasher.finalize().to_string(),
            total_size: self.stream_offset,
        })
    }

    pub fn seen_bytes(&self) -> usize {
        self.stream_offset
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StreamingCarAnalyzerError {
    #[error("received {0} bytes while still decoding the header which exceeds our allowed header sizes")]
    HeaderSegmentSizeExceeded(usize),

    #[error("received {0} bytes which exceeds our upper limit for an individual CAR upload")]
    MaxCarSizeExceeded(usize),
}
