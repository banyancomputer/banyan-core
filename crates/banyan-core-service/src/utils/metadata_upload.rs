use futures::{TryStream, TryStreamExt};
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::utils::car_buffer::CarBuffer;

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
