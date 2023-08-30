use blake3::Hasher;
use bytes::{Bytes, BytesMut};

const CAR_HEADER_UPPER_LIMIT: u64 = 16 * 1024 * 1024; // Limit car headers to 16MiB

const CAR_FILE_UPPER_LIMIT: u64 = 32 * 1024 * 1024 * 1024; // We limit individual CAR files to 32GiB
                                                             //
const CARV2_PRAGMA: &[u8] = &[0x0a, 0xa1, 0x67, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x02];

pub struct BlockMeta {
    cid: String,
    offset: u64,
    length: u64,
}

#[derive(Clone, Debug)]
enum CarState {
    Pragma, // 11 bytes
    CarV2Header, // 40 bytes
    CarV1Header { // variable length, collects roots
        data_start: u64,
        data_end: u64,
        index_start: u64,
    },
    Block { // advances to each block until we reach data_end
        block_start: u64,
        data_end: u64,
        index_start: u64,
    },
    Index, // once we're in the indexes here we don't care anymore
    Complete,
}

pub struct CarReport {
    integrity_hash: String,
    total_size: u64,
}

impl CarReport {
    pub fn integrity_hash(&self) -> &str {
        self.integrity_hash.as_str()
    }

    pub fn total_size(&self) -> u64 {
        self.total_size
    }
}

pub struct StreamingCarAnalyzer {
    buffer: BytesMut,
    state: CarState,
    stream_offset: u64,

    hasher: blake3::Hasher,
}

impl StreamingCarAnalyzer {
    pub fn add_chunk(&mut self, bytes: &Bytes) -> Result<(), StreamingCarAnalyzerError> {
        self.exceeds_buffer_limit(bytes.len() as u64)?;

        // todo: we don't need to copy all the data depending on the state we're in, we can skip
        // over some of it to save resources and abort earlier for invalid/excessive files, but for
        // now let's just copy it all in to save dev time...
        self.buffer.extend_from_slice(&bytes);

        Ok(())
    }

    fn exceeds_buffer_limit(&self, new_bytes: u64) -> Result<(), StreamingCarAnalyzerError> {
        let new_byte_total = self.stream_offset + new_bytes;

        if new_byte_total > CAR_FILE_UPPER_LIMIT {
            return Err(StreamingCarAnalyzerError::MaxCarSizeExceeded(new_byte_total));
        }

        Ok(())
    }

    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
            state: CarState::Pragma,
            stream_offset: 0,

            hasher: blake3::Hasher::new(),
        }
    }

    pub async fn next(&mut self) -> Result<Option<BlockMeta>, StreamingCarAnalyzerError> {
        loop {
            match &self.state {
                CarState::Pragma => {
                    if self.buffer.len() < 11 {
                        return Ok(None);
                    }

                    let pragma_bytes = self.buffer.split_to(11);
                    self.stream_offset += 11;

                    if &pragma_bytes[..] != CARV2_PRAGMA {
                        return Err(StreamingCarAnalyzerError::PragmaMismatch);
                    }

                    self.state = CarState::CarV2Header;
                }
                CarState::CarV2Header => {
                    if self.buffer.len() < 40 {
                        return Ok(None);
                    }

                    let _capability_bytes = self.buffer.split_to(std::mem::size_of::<u128>());

                    let data_start_bytes = self.buffer.split_to(std::mem::size_of::<u64>());
                    let data_start = u64::from_le_bytes(data_start_bytes[..].try_into().expect("the exact size"));

                    let data_size_bytes = self.buffer.split_to(std::mem::size_of::<u64>());
                    let data_size = u64::from_le_bytes(data_size_bytes[..].try_into().expect("the exact size"));

                    let index_start_bytes = self.buffer.split_to(std::mem::size_of::<u64>());
                    let index_start = u64::from_le_bytes(index_start_bytes[..].try_into().expect("the exact size"));

                    self.stream_offset += 40;

                    self.state = CarState::CarV1Header {
                        data_start,
                        data_end: data_start + data_size,
                        index_start,
                    };
                }
                CarState::CarV1Header { data_start, data_end, index_start } => {
                    // todo: read varint in buffer to find out how large the header is

                    //if header_size >= CAR_HEADER_UPPER_LIMIT {
                    //    return Err(StreamingCarAnalyzerError::HeaderSegmentSizeExceeded(header_size));
                    //}

                    // todo: decode dag-cbor inside of block
                    // todo: parse out expected roots and record them

                    return Ok(None);
                }
                CarState::Block { block_start, data_end, index_start } => {
                    // Skip to the beginning of the block if we're not already at the beginning of
                    // one
                    if self.stream_offset < *block_start {
                        let skippable_bytes = block_start - self.stream_offset;
                        let available_bytes = self.buffer.len() as u64;

                        let skipped_byte_count = available_bytes.min(skippable_bytes);
                        let _ = self.buffer.split_to(skipped_byte_count as usize);
                        self.stream_offset += skipped_byte_count;

                        // We didn't quite have enough data to make it to the beginning of the
                        // block
                        if self.stream_offset != *block_start {
                            return Ok(None);
                        }
                    }

                    // todo: if there are enough bytes available... get the length and CID of the block
                    // todo: if we get block data transition our state to the next block
                    // todo: if we get block data return Ok(Some(data))
                }
                _ => return Ok(None),
            }
        }
    }

    pub fn report(self) -> Result<CarReport, StreamingCarAnalyzerError> {
        Ok(CarReport {
            integrity_hash: self.hasher.finalize().to_string(),
            total_size: self.stream_offset,
        })
    }

    pub fn seen_bytes(&self) -> u64 {
        self.stream_offset
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StreamingCarAnalyzerError {
    #[error("received {0} bytes while still decoding the header which exceeds our allowed header sizes")]
    HeaderSegmentSizeExceeded(u64),

    #[error("received {0} bytes which exceeds our upper limit for an individual CAR upload")]
    MaxCarSizeExceeded(u64),

    #[error("received car file did not have the expected pragma")]
    PragmaMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_v2_header(chars: u128, data_offset: u64, data_size: u64, index_offset: u64) -> Bytes {
        let mut buffer = BytesMut::new();

        buffer.extend_from_slice(&chars.to_le_bytes());
        buffer.extend_from_slice(&data_offset.to_le_bytes());
        buffer.extend_from_slice(&data_size.to_le_bytes());
        buffer.extend_from_slice(&index_offset.to_le_bytes());

        assert_eq!(buffer.len(), 40);
        buffer.freeze()
    }

    #[test]
    fn test_streaming_lifecycle() {
        let mut sca = StreamingCarAnalyzer::new();
        assert_eq!(sca.state, CarState::Pragma);

        // No data shouldn't transition
        assert!(sca.next().expect("still valid").is_none());
        assert_eq!(sca.state, CarState::Pragma);

        // Some data but still not enough, shouldn't transition
        sca.add_chunk(&Bytes::from(&CARV2_PRAGMA[0..4])).unwrap();
        assert!(sca.next().expect("still valid").is_none()); // no blocks yet
        assert_eq!(sca.state, CarState::Pragma);

        // The rest of the Pragma should do the trick
        sca.add_chunk(&Bytes::from(&CARV2_PRAGMA[4..])).unwrap();
        assert!(sca.next().expect("still valid").is_none()); // no blocks yet
        assert_eq!(sca.state, CarState::CarV2Header);

        let mut v2_header = encode_v2_header(0, 55, 128, 200);

        // Some data but still not enough, shouldn't transition
        sca.add_chunk(&v2_header.split_to(17)).unwrap();
        assert!(sca.next().expect("still valid").is_none()); // no blocks yet
        assert_eq!(sca.state, CarState::CarV2Header);

        // The rest of the header
        sca.add_chunk(&v2_header).unwrap();
        assert!(sca.next().expect("still valid").is_none()); // no blocks yet
        assert_eq!(sca.state, CarState::CarV1Header { data_start: 55, data_end: 183, index_start: 200 });
    }
}
