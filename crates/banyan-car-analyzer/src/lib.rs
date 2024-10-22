use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bytes::{Bytes, BytesMut};

const CAR_HEADER_UPPER_LIMIT: u64 = 16 * 1024 * 1024; // Limit car headers to 16MiB

const CAR_FILE_UPPER_LIMIT: u64 = 32 * 1024 * 1024 * 1024; // We limit individual CAR files to 32GiB
                                                           //
const CARV2_PRAGMA: &[u8] = &[
    0x0a, 0xa1, 0x67, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x02,
];

#[derive(Debug, PartialEq)]
pub struct Block {
    cid: String,
    offset: u64,
    length: u64,
    data: Vec<u8>,
}

impl Block {
    pub fn cid(&self) -> &str {
        &self.cid
    }

    pub fn into_data(self) -> Vec<u8> {
        self.data
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
    BlockMeta {
        // advances to each block until we reach data_end
        block_start: u64,
        data_end: u64,
        index_start: u64,
    },
    BlockData {
        // advances to each block until we reach data_end
        data_start: u64,
        data_length: u64,
        cid: String,

        data_end: u64,
        index_start: u64,
    },
    Indexes {
        index_start: u64,
    }, // once we're in the indexes here we don't care anymore
    Complete,
}

pub struct CarReport {
    integrity_hash: String,
    total_size: u64,
    cids: Vec<String>,
}

impl CarReport {
    pub fn integrity_hash(&self) -> &str {
        self.integrity_hash.as_str()
    }

    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    pub fn cids(&self) -> &[String] {
        self.cids.as_slice()
    }
}

#[derive(Debug)]
pub struct StreamingCarAnalyzer {
    buffer: BytesMut,
    state: CarState,
    stream_offset: u64,
    cids: Vec<String>,
    hasher: blake3::Hasher,
}

impl Default for StreamingCarAnalyzer {
    fn default() -> Self {
        Self::new()
    }
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
            cids: Vec::new(),
            hasher: blake3::Hasher::new(),
        }
    }

    pub async fn next(&mut self) -> Result<Option<Block>, StreamingCarAnalyzerError> {
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
                    let data_start = *data_start;

                    // Skip any padding or whitespace until the beginning of our header
                    if self.stream_offset < data_start {
                        let skippable_bytes = data_start - self.stream_offset;
                        let available_bytes = self.buffer.len() as u64;

                        let skipped_byte_count = available_bytes.min(skippable_bytes);
                        let _ = self.buffer.split_to(skipped_byte_count as usize);
                        self.stream_offset += skipped_byte_count;

                        if self.stream_offset != data_start {
                            return Ok(None);
                        }
                    }

                    let hdr_len = match header_length {
                        Some(l) => *l,
                        None => match try_read_varint_u64(&self.buffer[..])? {
                            Some((length, bytes_read)) => {
                                *header_length = Some(length);

                                self.stream_offset += bytes_read;
                                let _ = self.buffer.split_to(bytes_read as usize);

                                length
                            }
                            None => return Ok(None),
                        },
                    };

                    if hdr_len >= CAR_HEADER_UPPER_LIMIT {
                        return Err(StreamingCarAnalyzerError::HeaderSegmentSizeExceeded(
                            hdr_len,
                        ));
                    }

                    // todo: decode dag-cbor inside of block
                    // todo: parse out expected roots and record them... can skip for now

                    // into the blocks!
                    self.state = CarState::BlockMeta {
                        block_start: self.stream_offset + hdr_len,
                        data_end: *data_end,
                        index_start: *index_start,
                    };
                }
                CarState::BlockMeta {
                    block_start,
                    data_end,
                    index_start,
                } => {
                    let block_start = *block_start;

                    // Skip any left over data and padding until we reach our goal
                    if self.stream_offset < block_start {
                        let skippable_bytes = block_start - self.stream_offset; // 171 - 72 = 99
                        let available_bytes = self.buffer.len() as u64; //

                        let skipped_byte_count = available_bytes.min(skippable_bytes);
                        let _ = self.buffer.split_to(skipped_byte_count as usize);
                        self.stream_offset += skipped_byte_count;

                        if self.stream_offset != block_start {
                            return Ok(None);
                        }
                    }

                    if block_start == *data_end {
                        self.state = CarState::Indexes {
                            index_start: *index_start,
                        };

                        continue;
                    }

                    // We very carefully don't advance the stream offset for the rest of this
                    // variant, the BlockData state will do that for us once we get the minimum
                    // meta data from the record.

                    let (blk_len, varint_len) = match try_read_varint_u64(&self.buffer[..])? {
                        Some(varint) => varint,
                        None => return Ok(None),
                    };

                    // A cid of length 49 is our standard one but we've also used a 97 length one.
                    // We'll try and decode once we hit that if not we'll wait for more to come
                    // in.
                    let minimum_cid_blocks = blk_len.min(49) as usize;
                    let cid_buffer = &self.buffer[(varint_len as usize)..];
                    if cid_buffer.len() < minimum_cid_blocks {
                        return Ok(None);
                    }

                    let cid_length = match cid_buffer[0] {
                        // old style CIDs
                        0x62 => 59,
                        // banyanfs CIDs
                        0x75 => 49,
                        _ => {
                            return Err(StreamingCarAnalyzerError::InvalidBlockCid(
                                self.stream_offset,
                            ))
                        }
                    };

                    let cid =
                        String::from_utf8(cid_buffer[..cid_length].to_vec()).map_err(|_| {
                            StreamingCarAnalyzerError::InvalidBlockCid(self.stream_offset)
                        })?;

                    self.cids.push(cid.clone());

                    // This might be the end of all data, we'll check once we reach the block_start
                    // offset
                    self.state = CarState::BlockData {
                        data_start: self.stream_offset + varint_len + cid_length as u64,
                        data_length: blk_len - cid_length as u64,
                        cid,

                        data_end: *data_end,
                        index_start: *index_start,
                    };
                }
                CarState::BlockData {
                    data_start,
                    data_length,
                    cid,

                    data_end,
                    index_start,
                } => {
                    let data_start = *data_start;
                    let data_length = *data_length;
                    let cid = cid.to_string();

                    // Skip any left over data and padding until we reach our goal
                    if self.stream_offset < data_start {
                        let skippable_bytes = data_start - self.stream_offset;
                        let available_bytes = self.buffer.len() as u64;

                        let skipped_byte_count = available_bytes.min(skippable_bytes);
                        let _ = self.buffer.split_to(skipped_byte_count as usize);
                        self.stream_offset += skipped_byte_count;

                        if self.stream_offset != data_start {
                            return Ok(None);
                        }
                    }

                    // Wait until we have the entire before continuing
                    if self.buffer.len() < data_length as usize {
                        println!(
                            "insufficient data available {} vs {}",
                            self.buffer.len(),
                            data_length
                        );
                        return Ok(None);
                    }

                    let data = self.buffer.split_to(data_length as usize).to_vec();
                    self.stream_offset += data.len() as u64;

                    // This might be the end of all data, we'll check once we reach the block_start
                    // offset
                    self.state = CarState::BlockMeta {
                        block_start: data_start + data_length,
                        data_end: *data_end,
                        index_start: *index_start,
                    };

                    return Ok(Some(Block {
                        cid,
                        offset: data_start,
                        length: data_length,
                        data,
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
            cids: self.cids,
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

    #[error("CID located at offset {0} was not valid")]
    InvalidBlockCid(u64),

    #[error("received {0} bytes which exceeds our upper limit for an individual CAR upload")]
    MaxCarSizeExceeded(u64),

    #[error("received car file did not have expected blake3 hash")]
    MismatchedHash,

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

fn try_read_varint_u64(buf: &[u8]) -> Result<Option<(u64, u64)>, StreamingCarAnalyzerError> {
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
            return Ok(Some((result, encoded_length as u64)));
        }
    }

    Ok(None)
}

pub fn quick_cid(data: &[u8]) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;

    let mut cid_bytes = Vec::with_capacity(36);

    cid_bytes.extend_from_slice(&[0x01, 0x55, 0x1e, 0x20]);
    cid_bytes.extend_from_slice(blake3::hash(data).as_bytes());

    let encoded = URL_SAFE_NO_PAD.encode(cid_bytes);

    format!("u{}", encoded)
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;

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

    #[test]
    fn test_varint_roundtrip() {
        let reference_numbers: &[(u64, u64)] =
            &[(0, 1), (100, 1), (1000, 2), (10000, 2), (100000000, 4)];
        for (num, size) in reference_numbers.iter() {
            let encoded_version = encode_varint_u64(*num).to_vec();
            assert_eq!(
                try_read_varint_u64(encoded_version.as_slice()).unwrap(),
                Some((*num, *size))
            );
        }

        assert_eq!(try_read_varint_u64(&[]).unwrap(), None);
    }

    #[tokio::test]
    #[ignore]
    async fn test_streaming_lifecycle() {
        let mut sca = StreamingCarAnalyzer::new();
        assert_eq!(sca.state, CarState::Pragma);

        // No data shouldn't transition
        assert!(sca.next().await.expect("still valid").is_none());
        assert_eq!(sca.stream_offset, 0);
        assert_eq!(sca.state, CarState::Pragma);
        assert_eq!(sca.buffer.len(), 0);

        // Some data but still not enough, shouldn't transition
        sca.add_chunk(&Bytes::from(&CARV2_PRAGMA[0..4])).unwrap();
        assert!(sca.next().await.expect("still valid").is_none()); // no blocks yet
        assert_eq!(sca.stream_offset, 0);
        assert_eq!(sca.state, CarState::Pragma);
        assert_eq!(sca.buffer.len(), 4);

        // The rest of the Pragma should do the trick
        sca.add_chunk(&Bytes::from(&CARV2_PRAGMA[4..])).unwrap();
        assert!(sca.next().await.expect("still valid").is_none()); // no blocks yet
        assert_eq!(sca.stream_offset, 11);
        assert_eq!(sca.state, CarState::CarV2Header);
        assert_eq!(sca.buffer.len(), 0);

        // data size is missing the size of the CID...
        let data_length = 1 + 99 + 1 + 36 + 57;
        let mut v2_header = encode_v2_header(0, 71, data_length, 285);

        // Some data but still not enough, shouldn't transition
        sca.add_chunk(&v2_header.split_to(17)).unwrap();
        assert!(sca.next().await.expect("still valid").is_none()); // no blocks yet
        assert_eq!(sca.stream_offset, 11);
        assert_eq!(sca.state, CarState::CarV2Header);
        assert_eq!(sca.buffer.len(), 17);

        // The rest of the header
        sca.add_chunk(&v2_header).unwrap();
        assert_eq!(sca.buffer.len(), 40);

        let car_v1_header = CarState::CarV1Header {
            data_start: 71,
            data_end: 71 + data_length,
            index_start: 285,
            header_length: None,
        };

        assert!(sca.next().await.expect("still valid").is_none()); // no blocks yet
        assert_eq!(sca.stream_offset, 51);
        assert_eq!(sca.state, car_v1_header); // this is taking one too many bytes...
        assert_eq!(sca.buffer.len(), 0);

        // We should automatically consume all of the padding and take us up to our first byte in
        // the car v1 header, which should also match the data_start value
        sca.add_chunk(&Bytes::from([0u8; 20].as_slice())).unwrap();
        assert_eq!(sca.buffer.len(), 20);

        assert!(sca.next().await.expect("still valid").is_none());
        assert_eq!(sca.stream_offset, 71);
        assert_eq!(sca.state, car_v1_header);
        assert_eq!(sca.buffer.len(), 0);

        // We're next going to advance our state until we can read the length of the header, we
        // don't care about the contents of the header so we're going to immediately start looking
        // for the first block before we get past the header contents.
        sca.add_chunk(&encode_varint_u64(99)).unwrap();
        assert_eq!(sca.buffer.len(), 1); // 1 byte

        // The parser will now know how long the header is, there is an intermediate state that is
        // hidden due to the loop where we have the length in the CarV1Header state, but since
        // we're not doing anything with that data we jump immediately to the first block...

        assert!(sca.next().await.expect("still valid").is_none());
        assert_eq!(sca.stream_offset, 72);

        let first_block = CarState::BlockMeta {
            block_start: 171,
            data_end: 71 + data_length,
            index_start: 71 + data_length + 20,
        };
        assert_eq!(sca.state, first_block);
        assert_eq!(sca.buffer.len(), 0);

        // Add in all the bytes that make up our header and advance to the start of our first block
        sca.add_chunk(&Bytes::from([0u8; 99].as_slice())).unwrap();
        assert_eq!(sca.buffer.len(), 99);

        assert!(sca.next().await.expect("still valid").is_none());
        assert_eq!(sca.stream_offset, 171);
        assert_eq!(sca.state, first_block);
        assert_eq!(sca.buffer.len(), 0);

        let block_data = b"some internal blockity block data, this is real I promise";
        // we'll use the RAW codec for our data...
        let block_cid = quick_cid(block_data);
        let cid_length = block_cid.as_bytes().len();

        let inner_block_size = (block_data.len() + cid_length) as u64;
        let length_bytes = encode_varint_u64(inner_block_size);
        let true_data_start = 171 + (length_bytes.len() + cid_length) as u64;

        sca.add_chunk(&length_bytes).unwrap();
        sca.add_chunk(&Bytes::from(block_cid.as_bytes().to_vec()))
            .unwrap();
        sca.add_chunk(&Bytes::from(block_data.to_vec())).unwrap();

        let next_meta = Some(Block {
            cid: block_cid,
            offset: true_data_start,
            length: block_data.len() as u64,
            data: block_data.to_vec(),
        });
        assert_eq!(sca.next().await.expect("still valid"), next_meta);
        assert_eq!(
            sca.state,
            CarState::BlockMeta {
                block_start: 265,
                data_end: 265,
                index_start: 285,
            }
        );
        assert_eq!(sca.stream_offset, 265);

        sca.add_chunk(&Bytes::from([0u8; 10].as_slice())).unwrap(); // take us into the padding past the data but before the indexes
        assert!(sca.next().await.expect("still valid").is_none()); // we're at the end of the data, this should transition to indexes
        assert_eq!(sca.state, CarState::Indexes { index_start: 285 });
        assert_eq!(sca.stream_offset, 275); // we read right up to the start of the virtual end
                                            // block

        // take us past the index so we can complete
        sca.add_chunk(&Bytes::from([0u8; 50].as_slice())).unwrap();
        assert!(sca.next().await.expect("still valid").is_none());
        assert_eq!(sca.state, CarState::Complete);

        let report = sca.report().unwrap();
        assert_eq!(report.total_size, 325);
    }

    pub(crate) fn quick_cid(data: &[u8]) -> String {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        use base64::Engine;

        let mut cid_bytes = Vec::with_capacity(36);

        cid_bytes.extend_from_slice(&[0x01, 0x55, 0x1e, 0x20]);
        cid_bytes.extend_from_slice(blake3::hash(data).as_bytes());

        let encoded = URL_SAFE_NO_PAD.encode(cid_bytes);

        format!("u{}", encoded)
    }
}
