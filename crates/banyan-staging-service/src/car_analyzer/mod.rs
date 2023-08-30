use std::collections::HashSet;

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

    NeedData(usize, Box<CarState>),
    SkipUntil(usize, Box<CarState>),

    Header,
    BlockStart,
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

    // todo: switch types to fixed byte string...
    known_roots: HashSet<String>,
}

impl StreamingCarAnalyzer {
    pub fn add_chunk(&mut self, bytes: &Bytes) -> Result<(), StreamingCarAnalyzerError> {
        self.exceeds_buffer_limit(bytes.len())?;

        match &self.state {
            CarState::SkipUntil(desired_offset, next_state) => {
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

                self.state = *next_state.clone();
            }
            // We save ourselves the necessity of an extra loop later on if we just advance this
            // state here when we already know if its changing
            CarState::NeedData(desired_bytes, next_state) => {
                self.buffer.extend_from_slice(&bytes);

                if *desired_bytes <= bytes.len() {
                    self.state = *next_state.clone();
                }
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

        if matches!(self.state, CarState::Init) && new_byte_total > CAR_HEADER_UPPER_LIMIT {
            return Err(StreamingCarAnalyzerError::HeaderSegmentSizeExceeded(new_byte_total));
        }

        // There is case here that isn't being covered, when we're in the NeedsData state and our
        // target is the header we should also be checking to ensure our thresholds won't be
        // exceeded. Rather than perform that check here I'm going to perform that check when
        // transitioning to that state.
        //
        // I might actually just want to have a sub-parser responsible for the header to
        // disambiguate the states...

        Ok(())
    }

    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
            state: CarState::Init,
            stream_offset: 0,

            known_roots: HashSet::new(),
            hasher: blake3::Hasher::new(),
        }
    }

    pub async fn next(&mut self) -> Result<Option<BlockMeta>, StreamingCarAnalyzerError> {
        use CarState::*;

        //match self.state {
        //    _ => (),
        //}

        Ok(None)
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
