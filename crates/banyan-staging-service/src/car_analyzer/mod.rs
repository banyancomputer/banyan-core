use std::collections::HashSet;

use blake3::Hasher;
use bytes::{Bytes, BytesMut};

pub struct BlockMeta {
    cid: String,
    offset: usize,
    length: usize,
}

#[derive(Debug)]
enum CarState {
    Header,
    BlockScanning(usize),
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
    pub fn add_chunk(&mut self, bytes: &Bytes) {
        self.hasher.update(&bytes);

        if let CarState::BlockScanning(desired_offset) = self.state {
            // When scanning for blocks, we only care about the first few bytes of each block,
            // we don't need to copy the actual block's data into our buffer
            if (self.stream_offset + bytes.len()) < desired_offset {
                self.state = CarState::BlockScanning(self.stream_offset - bytes.len());
                self.stream_offset += bytes.len();

                return;
            }

            // This new data brings us up to the beginning of a new block, only copy in the
            // bytes that are relevant for us. This is technically more than we need, it can be
            // optimized in the future.
            let ignored_bytes = desired_offset - self.stream_offset;
            self.buffer.extend_from_slice(&bytes[ignored_bytes..]);
            self.stream_offset += ignored_bytes;
            self.state = CarState::BlockStart;
        }

        match self.state {
            CarState::BlockScanning(_) => unreachable!("we already returned or transitioned by now"),
            _ => {
                self.buffer.extend_from_slice(&bytes);
            }
        }
    }

    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
            state: CarState::Header,
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
    #[error("placeholder")]
    Placeholder,
}
