use std::{collections::HashSet, hash::Hash, io, marker::PhantomData};

use crate::{expression_discriminant, io_error, Expression, FromPath, Schema};

pub struct HashSetExpression<T: Schema + Send + Sync + Eq + Hash>(Vec<u32>, PhantomData<T>);

impl<T: Schema + Send + Sync + Eq + Hash> Clone for HashSetExpression<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T: Schema + Send + Sync + Eq + Hash> Expression for HashSetExpression<T> {
    type Target = HashSet<T>;

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

impl<T: Schema + Send + Sync + Eq + Hash> FromPath for HashSetExpression<T> {
    fn from_path(path: Vec<u32>) -> Self {
        Self(path, PhantomData)
    }
}
