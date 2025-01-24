use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{io_error, OptionExpression, Schema};

impl<S: Schema + Send + Sync> Schema for Option<S> {
    type Expression = OptionExpression<S>;

    fn write_schema(
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        async {
            write.write_u8(1).await?;
            write.write_u32(2).await?;
            <()>::write_schema(write).await?;
            S::write_schema(write).await?;

            Ok(())
        }
    }

    fn write_value(
        &self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        async move {
            match self {
                None => write.write_u32(0).await?,
                Some(value) => {
                    write.write_u32(1).await?;
                    value.write_value(write).await?;
                }
            }

            Ok(())
        }
    }

    fn read_value(
        read: &mut (impl tokio::io::AsyncReadExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<Self>> + Send {
        async {
            match read.read_u32().await? {
                0 => Ok(None),
                1 => Ok(Some(S::read_value(read).await?)),
                _ => Err(io_error!(
                    InvalidData,
                    "invalid discriminant in value for a sum value"
                )),
            }
        }
    }
}
