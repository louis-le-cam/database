use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{
    expression_discriminant, io_error, schema_discriminant, Expression, PathExpression, Schema,
};

impl<T: Schema + Send + Sync> Schema for Vec<T> {
    type Expression = PathExpression<Vec<T>>;

    fn write_schema(
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        async {
            write.write_u8(schema_discriminant::LIST).await?;
            T::write_schema(write).await?;

            Ok(())
        }
    }

    fn write_value(
        &self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        async move {
            write
                .write_u32(self.len().try_into().map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "list value length doesn't fit into a 32 bit unsigned integer",
                    )
                })?)
                .await?;

            for value in self {
                value.write_value(write).await?;
            }

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
                    "list value length doesn't fit into a pointer sized unsigned integer",
                )
            })?;

            let mut values = Vec::new();
            values.try_reserve(length).map_err(|_| {
                io_error!(OutOfMemory, "allocation of memory for list values failed")
            })?;

            for _ in 0..length {
                values.push(T::read_value(read).await?);
            }

            Ok(values)
        }
    }
}

impl<T: Expression> Expression for Vec<T>
where
    T::Target: Send + Sync,
{
    type Target = Vec<T::Target>;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async move {
            write.write_u8(expression_discriminant::LIST).await?;

            write
                .write_u32(self.len().try_into().map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "list expression length doesn't fit into a 32 bit unsigned integer",
                    )
                })?)
                .await?;

            for value in self {
                value.write(write).await?;
            }

            Ok(())
        }
    }
}
