use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{io_error, PathExpression, Schema, SchemaNode};

impl Schema for String {
    type Expression = PathExpression<String>;

    fn write_schema(
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        SchemaNode::String.write(write)
    }

    fn write_value(
        &self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        async {
            write
                .write_u32(self.len().try_into().map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "string value length doesn't fit into a 32 bit unsigned integer",
                    )
                })?)
                .await?;

            write.write_all(self.as_bytes()).await?;

            Ok(())
        }
    }

    fn read_value(
        read: &mut (impl tokio::io::AsyncReadExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<Self>> + Send {
        async {
            let length: usize = read.read_u32().await?.try_into().map_err(|_| {
                io_error!(
                    OutOfMemory,
                    "string value length doesn't fit into a pointer sized unsigned integer",
                )
            })?;

            let mut string_bytes = Vec::new();
            string_bytes.try_reserve(length).map_err(|_| {
                io_error!(OutOfMemory, "allocation of memory for string value failed")
            })?;
            string_bytes.extend((0..length).map(|_| 0));

            read.read_exact(&mut string_bytes).await?;

            String::from_utf8(string_bytes)
                .map_err(|_| io_error!(InvalidData, "allocation of memory for string value failed"))
        }
    }
}
