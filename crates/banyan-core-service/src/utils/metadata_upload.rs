use futures::{TryStream, TryStreamExt};
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::utils::car_buffer::CarBuffer;

pub async fn handle_metadata_upload<S>(
    mut stream: S,
    writer: &mut Box<dyn AsyncWrite + Unpin + Send>,
) -> Result<(String, u64), ()>
where
    S: TryStream<Ok = bytes::Bytes> + Unpin,
    S::Error: std::error::Error,
{
    let mut car_buffer = CarBuffer::new();
    let mut hasher = blake3::Hasher::new();
    let mut bytes_written = 0;

    while let Some(chunk) = stream.try_next().await.transpose() {
        let chunk = chunk.expect("an available chunk (todo remove this)");

        hasher.update(&chunk);
        car_buffer.add_chunk(&chunk);
        bytes_written += chunk.len();

        writer
            .write_all(&chunk)
            .await
            .expect("the write to succeed (todo remove this)");
    }

    let hash = hasher.finalize();

    Ok((hash.to_string(), bytes_written as u64))
}

/// Round a number of bytes to the nearest 100 MiB
/// If the number of bytes is exactly between two 100 MiB boundaries, round up
pub fn round_to_nearest_100_mib(bytes: u64) -> u64 {
    const ONE_HUNDRED_MIB: u64 = 100 * 1024 * 1024; // 100 * 1 MiB in bytes

    let remainder = bytes % ONE_HUNDRED_MIB;
    let quotient = bytes / ONE_HUNDRED_MIB;

    if remainder == 0 {
        bytes
    } else {
        (quotient + 1) * ONE_HUNDRED_MIB
    }
}
