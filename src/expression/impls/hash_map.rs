use std::{collections::HashMap, hash::Hash, io, marker::PhantomData};

use crate::{expression_discriminant, io_error, Expression, FromPath, Schema};

pub struct HashMapExpression<K: Schema + Send + Sync + Eq + Hash, V: Schema + Send + Sync>(
    Vec<u32>,
    PhantomData<(K, V)>,
);

impl<K: Schema + Send + Sync + Eq + Hash, V: Schema + Send + Sync> Clone
    for HashMapExpression<K, V>
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<K: Schema + Send + Sync + Eq + Hash, V: Schema + Send + Sync> Expression
    for HashMapExpression<K, V>
{
    type Target = HashMap<K, V>;

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

impl<K: Schema + Send + Sync + Eq + Hash, V: Schema + Send + Sync> FromPath
    for HashMapExpression<K, V>
{
    fn from_path(path: Vec<u32>) -> Self {
        Self(path, PhantomData)
    }
}
