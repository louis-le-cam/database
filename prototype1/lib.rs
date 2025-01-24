use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub mod pipeline;
pub mod request;
pub mod schema;
pub mod value;

pub async fn write_path(
    path: &[u32],
    write: &mut (impl AsyncWriteExt + Unpin),
) -> std::io::Result<()> {
    write.write_u32(path.len().try_into().unwrap()).await?;

    for segment in path {
        write.write_u32(*segment).await?;
    }

    Ok(())
}

pub async fn read_path(read: &mut (impl AsyncReadExt + Unpin)) -> std::io::Result<Vec<u32>> {
    let length = read.read_u32().await?;
    let mut segments = Vec::with_capacity(length as usize);

    for _ in 0..length {
        segments.push(read.read_u32().await?);
    }

    Ok(segments)
}
