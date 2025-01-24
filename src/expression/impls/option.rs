use std::{io, marker::PhantomData};

use crate::{expression_discriminant, io_error, Expression, FromPath, Schema};

pub struct OptionExpression<S: Schema + Send + Sync>(Vec<u32>, PhantomData<S>);

impl<S: Schema + Send + Sync> Expression for OptionExpression<S> {
    type Target = bool;

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

impl<S: Schema + Send + Sync> FromPath for OptionExpression<S> {
    fn from_path(path: Vec<u32>) -> Self {
        Self(path, PhantomData)
    }
}
