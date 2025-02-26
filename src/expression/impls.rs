use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, Expression, Schema};

impl<S: Schema> Expression for S {
    type Target = <S::Expression as Expression>::Target;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async move {
            write.write_u8(expression_discriminant::VALUE).await?;

            S::write_schema(write).await?;
            self.write_value(write).await
        }
    }
}

impl Expression for &str {
    type Target = String;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async move {
            write.write_u8(expression_discriminant::VALUE).await?;

            String::write_schema(write).await?;
            // TODO: this allocation is useless but it's more convenient
            self.to_string().write_value(write).await
        }
    }
}
