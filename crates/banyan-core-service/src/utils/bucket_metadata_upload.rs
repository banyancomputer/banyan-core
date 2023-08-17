use futures::{TryStream, TryStreamExt};
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::utils::car_buffer::CarBuffer;

pub async fn handle_bucket_metadata_upload<S>(
    mut stream: S,
    writer: &mut Box<dyn AsyncWrite + Unpin + Send>,
) -> Result<(String, usize), ()>
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

        //match car_buffer.parse() {
        //    Ok(Some(_md)) => {
        //        // TODO: we have our metadata, do any validation we need to here
        //    }
        //    Ok(None) => (),
        //    Err(err) => {
        //        tracing::error!("received car buffer error: {err}");
        //        return Err(());
        //    }
        //}

        writer
            .write_all(&chunk)
            .await
            .expect("the write to succeed (todo remove this)");
    }

    let hash = hasher.finalize();

    Ok((hash.to_string(), bytes_written))
}