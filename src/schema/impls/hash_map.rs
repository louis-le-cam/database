use std::{collections::HashMap, future::Future, hash::Hash, io};

use tokio::io::AsyncWriteExt;

use crate::{
    expression_discriminant, io_error, schema_discriminant, Expression, HashMapExpression, Schema,
};

impl<K: Schema + Send + Sync + Eq + Hash, V: Schema + Send + Sync> Schema for HashMap<K, V> {
    type Expression = HashMapExpression<K, V>;

    fn write_schema(
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        async {
            write.write_u8(schema_discriminant::LIST).await?;
            write.write_u8(schema_discriminant::PRODUCT).await?;
            write.write_u32(2).await?;
            K::write_schema(write).await?;
            V::write_schema(write).await?;

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

            for (key, value) in self {
                key.write_value(write).await?;
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

            let mut values = HashMap::new();
            values.try_reserve(length).map_err(|_| {
                io_error!(OutOfMemory, "allocation of memory for list values failed")
            })?;

            for _ in 0..length {
                let key = K::read_value(read).await?;
                let value = V::read_value(read).await?;

                if values.insert(key, value).is_some() {
                    return Err(io_error!(InvalidData, "hashmap keys are not unique"));
                };
            }

            Ok(values)
        }
    }
}

// TODO: find a way to pass hashmap containing expressions in query
impl<K: Schema + Send + Sync + Eq + Hash, V: Schema + Send + Sync> Expression for HashMap<K, V> {
    type Target = HashMap<K, V>;

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

            for (key, value) in self {
                write.write_u8(expression_discriminant::VALUE).await?;
                <(K, V)>::write_schema(write).await?;
                (key, value).write_value(write).await?;
            }

            Ok(())
        }
    }
}
