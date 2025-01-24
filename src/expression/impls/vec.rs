use std::{io, marker::PhantomData};

use crate::{expression_discriminant, io_error, Expression, FromPath, Schema};

pub struct VecExpression<T: Schema + Send + Sync>(Vec<u32>, PhantomData<T>);

impl<T: Schema + Send + Sync> VecExpression<T> {
    pub fn get(&self, index: u32) -> T::Expression {
        T::Expression::from_path(self.0.iter().copied().chain([index]).collect())
    }
}

impl<T: Schema + Send + Sync> Expression for VecExpression<T> {
    type Target = Vec<T>;

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

impl<T: Schema + Send + Sync> FromPath for VecExpression<T> {
    fn from_path(path: Vec<u32>) -> Self {
        Self(path, PhantomData)
    }
}
