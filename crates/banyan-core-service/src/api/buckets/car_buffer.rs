use bytes::Bytes;

// We only look through the first MiB for the keys section. It's considered badly formed if not
// found in that window.
const SCANNED_HEADER_CAPACITY: usize = 1_048_576;

struct HeaderBuffer {
    state: BufferState,
}

enum BufferState {
    Buffered(Vec<u8>),
    Empty,
    FirstChunk(Bytes),
}
