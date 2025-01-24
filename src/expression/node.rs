use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::io_error;

pub enum ExpressionNode {
    Path(Vec<u32>),
}

impl ExpressionNode {
    fn discriminant(&self) -> u8 {
        match self {
            ExpressionNode::Path(_) => 0,
        }
    }

    pub async fn read(read: &mut (impl AsyncReadExt + Unpin)) -> io::Result<Self> {
        let discriminant = read.read_u8().await?;

        let node = match discriminant {
            0 => {
                let length: usize = read.read_u32().await?.try_into().map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "path expression length doesn't fit into a pointer sized unsigned integer",
                    )
                })?;

                let mut path = Vec::new();
                path.try_reserve(length).map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "allocation of memory for path expression failed"
                    )
                })?;

                for _ in 0..length {
                    path.push(read.read_u32().await?);
                }

                Self::Path(path)
            }
            _ => {
                return Err(io_error!(
                    InvalidData,
                    "invalid discriminant while parsing expression node",
                ));
            }
        };

        debug_assert_eq!(node.discriminant(), discriminant);

        Ok(node)
    }

    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        write.write_u8(self.discriminant()).await?;

        match self {
            ExpressionNode::Path(segments) => {
                write
                    .write_u32(segments.len().try_into().map_err(|_| {
                        io_error!(
                            OutOfMemory,
                            "path expression length doesn't fit into a 32 bit unsigned integer",
                        )
                    })?)
                    .await?;

                for segment in segments {
                    write.write_u32(*segment).await?;
                }
            }
        }

        Ok(())
    }
}
