use std::{io, marker::PhantomData};

use crate::{expression_discriminant, io_error, Expression, FromPath, Key, Schema, SlotMap};

pub struct SlotMapExpression<K: Key + Send + Sync, T: Schema + Send + Sync>(
    Vec<u32>,
    PhantomData<(K, T)>,
);

impl<K: Key + Send + Sync, T: Schema + Send + Sync> Clone for SlotMapExpression<K, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<K: Key + Send + Sync, T: Schema + Send + Sync> Expression for SlotMapExpression<K, T> {
    type Target = SlotMap<K, T>;

    async fn write(self, write: &mut (impl tokio::io::AsyncWriteExt + Unpin)) -> io::Result<()> {
        write.write_u8(expression_discriminant::PATH).await?;
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

impl<K: Key + Send + Sync, T: Schema + Send + Sync> FromPath for SlotMapExpression<K, T> {
    fn from_path(path: Vec<u32>) -> Self {
        Self(path, PhantomData)
    }
}
