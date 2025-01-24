use std::io;

use crate::{io_error, Expression, FromPath};

pub struct Uint32Expression(Vec<u32>);

impl Expression for Uint32Expression {
    type Target = u32;

    async fn write(self, write: &mut (impl tokio::io::AsyncWriteExt + Unpin)) -> io::Result<()> {
        write.write_u8(0).await?;
        write
            .write_u32(self.0.len().try_into().map_err(|_| {
                io_error!(
                    OutOfMemory,
                    "path expression length doesn't fit into a 32 bit unsigned integer",
                )
            })?)
            .await?;

        for segment in &self.0 {
            write.write_u32(*segment).await?;
        }

        Ok(())
    }
}

impl FromPath for Uint32Expression {
    fn from_path(path: Vec<u32>) -> Self {
        Self(path)
    }
}
