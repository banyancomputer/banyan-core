use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use blake3::Hasher;
use bytes::{BufMut, Bytes, BytesMut};
use cid::Cid;

const CAR_HEADER_UPPER_LIMIT: u64 = 16 * 1024 * 1024; // Limit car headers to 16MiB

const CAR_FILE_UPPER_LIMIT: u64 = 32 * 1024 * 1024 * 1024; // We limit individual CAR files to 32GiB
                                                           //
const CARV2_PRAGMA: &[u8] = &[
    0x0a, 0xa1, 0x67, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x02,
];

#[derive(Debug, PartialEq)]
pub struct BlockMeta {
    cid: Cid,
    offset: u64,
    length: u64,
}

impl BlockMeta {
    pub fn cid(&self) -> &Cid {
        &self.cid
    }

    pub fn length(&self) -> u64 {
        self.length
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CarState {
    Pragma,      // 11 bytes
    CarV2Header, // 40 bytes
    CarV1Header {
        // variable length, collects roots
        data_start: u64,
        data_end: u64,
        index_start: u64,

        header_length: Option<u64>,
    },
    Block {
        // advances to each block until we reach data_end
        block_start: u64,
        data_end: u64,
        index_start: u64,

        block_length: Option<u64>,
    },
    Indexes {
        index_start: u64,
    }, // once we're in the indexes here we don't care anymore
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

#[derive(Debug)]
pub struct StreamingCarAnalyzer {
    buffer: BytesMut,
    state: CarState,
    stream_offset: u64,

    hasher: blake3::Hasher,
}

impl StreamingCarAnalyzer {
    pub fn add_chunk(&mut self, bytes: &Bytes) -> Result<(), StreamingCarAnalyzerError> {
        self.exceeds_buffer_limit(bytes.len() as u64)?;

        // Don't bother copying data in once we're done analyzing the contents
        if matches!(self.state, CarState::Indexes { .. } | CarState::Complete) {
            self.stream_offset += bytes.len() as u64;
            return Ok(());
        }

        // todo: there are more states where we can avoid copying here such as
        self.buffer.extend_from_slice(bytes);

        Ok(())
    }

    fn exceeds_buffer_limit(&self, new_bytes: u64) -> Result<(), StreamingCarAnalyzerError> {
        let new_byte_total = self.stream_offset + new_bytes;

        if new_byte_total > CAR_FILE_UPPER_LIMIT {
            return Err(StreamingCarAnalyzerError::MaxCarSizeExceeded(
                new_byte_total,
            ));
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
            match &mut self.state {
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
                    let data_start = u64::from_le_bytes(
                        data_start_bytes[..].try_into().expect("the exact size"),
                    );

                    let data_size_bytes = self.buffer.split_to(std::mem::size_of::<u64>());
                    let data_size =
                        u64::from_le_bytes(data_size_bytes[..].try_into().expect("the exact size"));

                    let index_start_bytes = self.buffer.split_to(std::mem::size_of::<u64>());
                    let index_start = u64::from_le_bytes(
                        index_start_bytes[..].try_into().expect("the exact size"),
                    );

                    self.stream_offset += 40;

                    let data_end = data_start + data_size;
                    if data_end > CAR_FILE_UPPER_LIMIT {
                        return Err(StreamingCarAnalyzerError::MaxCarSizeExceeded(data_end));
                    }

                    if index_start > CAR_FILE_UPPER_LIMIT {
                        return Err(StreamingCarAnalyzerError::MaxCarSizeExceeded(index_start));
                    }

                    self.state = CarState::CarV1Header {
                        data_start,
                        data_end,
                        index_start,

                        header_length: None,
                    };
                }
                CarState::CarV1Header {
                    data_start,
                    data_end,
                    index_start,
                    ref mut header_length,
                } => {
                    let header_length = match header_length {
                        Some(hl) => *hl,
                        None => match try_read_varint_u64(&mut self.buffer)? {
                            Some((hl, byte_len)) => {
                                *header_length = Some(hl);
                                self.stream_offset += byte_len;
                                hl
                            }
                            None => return Ok(None),
                        },
                    };

                    if header_length >= CAR_HEADER_UPPER_LIMIT {
                        return Err(StreamingCarAnalyzerError::HeaderSegmentSizeExceeded(
                            header_length,
                        ));
                    }

                    // todo: decode dag-cbor inside of block
                    // todo: parse out expected roots and record them... can skip for now

                    // into the blocks!
                    // note: we're implicitly skipping the padding here
                    self.state = CarState::Block {
                        block_start: *data_start + header_length,
                        data_end: *data_end,
                        index_start: *index_start,

                        block_length: None,
                    };
                }
                CarState::Block {
                    block_start,
                    data_end,
                    index_start,
                    ref mut block_length,
                } => {
                    let block_start = *block_start;

                    // Skip to the beginning of the block if we're not already at the beginning of
                    // one
                    if self.stream_offset < block_start {
                        let skippable_bytes = block_start - self.stream_offset;
                        let available_bytes = self.buffer.len() as u64;

                        let skipped_byte_count = available_bytes.min(skippable_bytes);
                        let _ = self.buffer.split_to(skipped_byte_count as usize);
                        self.stream_offset += skipped_byte_count;

                        // We didn't quite have enough data to make it to the beginning of the
                        // block
                        if self.stream_offset != block_start {
                            return Ok(None);
                        }
                    }

                    // It would be more optimal to have a streaming state we could move into once
                    // we've extracted the CID that just skips over the data until we get to the
                    // end of this. Doing this here results in unecessary amounts of copying that
                    // could be cleaned up later.
                    if block_start == *data_end {
                        self.state = CarState::Indexes {
                            index_start: *index_start,
                        };
                        return Ok(None);
                    }
                    let block_length = match block_length {
                        Some(bl) => *bl,
                        None => match try_read_varint_u64(&mut self.buffer)? {
                            Some((bl, byte_len)) => {
                                *block_length = Some(bl);
                                self.stream_offset += byte_len;
                                bl
                            }
                            None => return Ok(None),
                        },
                    };

                    // 64-bytes is the longest reasonable CID we're going to care about it. We're
                    // going to wait until we have that much then try and decode the CID from
                    // there. The edge case here is if the total block length (CID included) is
                    // less than 64-bytes we'll just wait for the entire block. The CID has to be
                    // included and we'll decode it from there just as neatly.
                    let minimum_cid_blocks = block_length.min(64) as usize;
                    if self.buffer.len() < minimum_cid_blocks {
                        return Ok(None);
                    }
                    let cid = Cid::read_bytes(&self.buffer[..minimum_cid_blocks]).unwrap();
                    self.stream_offset += cid.encoded_len() as u64;
                    // This might be the end of all data, we'll check once we reach the block_start
                    // offset
                    self.state = CarState::Block {
                        block_start: block_start + block_length as u64,
                        data_end: *data_end,
                        index_start: *index_start,
                        block_length: None,
                    };

                    return Ok(Some(BlockMeta {
                        cid,
                        offset: block_start,
                        length: block_length,
                    }));
                }
                CarState::Indexes { index_start } => {
                    // we don't actually care about the indexes right now so I'm going to use this
                    // just as a convenient place to drain our buffer
                    self.stream_offset += self.buffer.len() as u64;
                    self.buffer.clear();

                    // We do want to make sure we at least get to the indexes...
                    if self.stream_offset >= *index_start {
                        self.state = CarState::Complete;
                    }

                    return Ok(None);
                }
                CarState::Complete => return Ok(None),
            }
        }
    }

    pub fn report(self) -> Result<CarReport, StreamingCarAnalyzerError> {
        if !matches!(self.state, CarState::Complete) {
            return Err(StreamingCarAnalyzerError::IncompleteData);
        }

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
    #[error(
        "received {0} bytes while still decoding the header which exceeds our allowed header sizes"
    )]
    HeaderSegmentSizeExceeded(u64),

    #[error("parser wasn't finished with the data stream before it ended")]
    IncompleteData,

    #[error("received {0} bytes which exceeds our upper limit for an individual CAR upload")]
    MaxCarSizeExceeded(u64),

    #[error("received car file did not have the expected pragma")]
    PragmaMismatch,

    #[error("a varint in the car file was larger than our acceptable value")]
    ValueToLarge,
}

impl IntoResponse for StreamingCarAnalyzerError {
    fn into_response(self) -> Response {
        let err_msg = serde_json::json!({ "msg": self.to_string() });
        (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
    }
}

// We need to account for extra bytes here due to the encoding, for every 7 bits we add an extra 1
// bit or ceil(64 / 7) + 64 = 74 bits. 74 bits pack into 10 bytes so that is the maximum number of
// bytes we care about.
const U64_MAX_ENCODED_LENGTH: usize = 10;

fn try_read_varint_u64(
    buf: &mut BytesMut,
) -> Result<Option<(u64, u64)>, StreamingCarAnalyzerError> {
    let mut result: u64 = 0;

    // The length check doesn't make this loop very efficient but it should be sufficient for now
    for i in 0..U64_MAX_ENCODED_LENGTH {
        // We don't have enough data
        if buf.len() <= i {
            return Ok(None);
        }

        result |= u64::from(buf[i] & 0b0111_1111) << (i * 7);

        // The leftmost bit being cleared indicates we're done with the decoding
        if buf[i] & 0b1000_0000 == 0 {
            let encoded_length = i + 1;
            let _ = buf.split_to(encoded_length);
            return Ok(Some((result, encoded_length as u64)));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    use cid::multihash::{Code, MultihashDigest};
    use cid::Cid;

    fn encode_v2_header(chars: , data_offset: u64, data_size: u64, index_offset: u64) -> Bytes {
        let mut buffer = BytesMut::new();

        buffer.extend_from_slice(&chars.to_le_bytes());
        buffer.extend_from_slice(&data_offset.to_le_bytes());
        buffer.extend_from_slice(&data_size.to_le_bytes());
        buffer.extend_from_slice(&index_offset.to_le_bytes());

        assert_eq!(buffer.len(), 40);
        buffer.freeze()
    }

    fn encode_varint_u64(mut val: u64) -> Bytes {
        let mut bytes = BytesMut::new();

        loop {
            let mut current_byte = (val & 0b0111_1111) as u8; // take the lower 7 bits
            val >>= 7; // shift them away

            if val > 0 {
                // This isn't the last byte, set the high bit
                current_byte |= 0b1000_0000;
            }

            // append our current byte to the byte list (this is doing the MSB to LSB conversion)
            bytes.put_u8(current_byte);

            // if nothing is remaining drop out of the loop
            if val == 0 {
                break;
            }
        }

        bytes.freeze()
    }

    #[tokio::test]
    async fn test_streaming_lifecycle() {
        let mut sca = StreamingCarAnalyzer::new();
        assert_eq!(sca.state, CarState::Pragma);

        // No data shouldn't transition
        assert!(sca.next().await.expect("still valid").is_none());
        assert_eq!(sca.state, CarState::Pragma);

        // Some data but still not enough, shouldn't transition
        sca.add_chunk(&Bytes::from(&CARV2_PRAGMA[0..4])).unwrap();
        assert!(sca.next().await.expect("still valid").is_none()); // no blocks yet
        assert_eq!(sca.state, CarState::Pragma);

        // The rest of the Pragma should do the trick
        sca.add_chunk(&Bytes::from(&CARV2_PRAGMA[4..])).unwrap();
        assert!(sca.next().await.expect("still valid").is_none()); // no blocks yet
        assert_eq!(sca.state, CarState::CarV2Header);

        let mut v2_header = encode_v2_header(0, 172, 93, 290);

        // Some data but still not enough, shouldn't transition
        sca.add_chunk(&v2_header.split_to(17)).unwrap();
        assert!(sca.next().await.expect("still valid").is_none()); // no blocks yet
        assert_eq!(sca.state, CarState::CarV2Header);

        // The rest of the header
        sca.add_chunk(&v2_header).unwrap();
        assert!(sca.next().await.expect("still valid").is_none()); // no blocks yet
        assert_eq!(
            sca.state,
            CarState::CarV1Header {
                data_start: 172,
                data_end: 265,
                index_start: 290,
                header_length: None
            }
        );

        sca.add_chunk(&encode_varint_u64(100)).unwrap();
        sca.add_chunk(&Bytes::from([0u8; 100].as_slice())).unwrap(); // we don't actually inspect the header data here yet
        sca.add_chunk(&Bytes::from([0u8; 20].as_slice())).unwrap(); // and the padding we calculated into the hardcoded numbers
        assert!(sca.next().await.expect("still valid").is_none()); // no blocks yet... but we're right on the edge of one!
        assert_eq!(
            sca.state,
            CarState::Block {
                block_start: 172,
                data_end: 265,
                index_start: 290,
                block_length: None
            }
        );
        assert_eq!(sca.stream_offset, 172); // It'll take us right up to next point we're looking for

        let block_data = b"some internal blockity block data, this is real I promise";
        // we'll use the RAW codec for our data...
        let block_cid = Cid::new_v1(0x55, Code::Sha3_256.digest(block_data));

        sca.add_chunk(&encode_varint_u64(
            (block_data.len() + block_cid.encoded_len()) as u64,
        ))
        .unwrap();
        sca.add_chunk(&Bytes::from(block_cid.to_bytes())).unwrap();
        sca.add_chunk(&Bytes::from(block_data.to_vec())).unwrap();

        let next_meta = Some(BlockMeta {
            cid: block_cid,
            offset: 172,
            length: 93,
        });
        assert_eq!(sca.next().await.expect("still valid"), next_meta);
        assert_eq!(
            sca.state,
            CarState::Block {
                block_start: 265,
                data_end: 265,
                index_start: 290,
                block_length: None
            }
        );
        assert_eq!(sca.stream_offset, 209); // we've only actually read the Cid, the data is still
                                            // in the buffer

        sca.add_chunk(&Bytes::from([0u8; 10].as_slice())).unwrap(); // take us into the padding past the data but before the indexes
        assert!(sca.next().await.expect("still valid").is_none()); // we're at the end of the data, this should transition to indexes
        assert_eq!(sca.state, CarState::Indexes { index_start: 290 });
        assert_eq!(sca.stream_offset, 265); // we read right up to the start of the virtual end
                                            // block

        // take us past the index so we can complete
        sca.add_chunk(&Bytes::from([0u8; 30].as_slice())).unwrap();
        assert!(sca.next().await.expect("still valid").is_none());
        assert_eq!(sca.state, CarState::Complete);

        let report = sca.report().unwrap();
        assert_eq!(report.total_size, 342);
    }
}
