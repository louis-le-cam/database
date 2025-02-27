use std::{future::Future, io, time::Duration};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, Expression, Schema};

macro_rules! impl_expr {
    ($($({$($gen:tt)*})? $type:ty)+) => {$(
        impl $(<$($gen)*>)? Expression for $type {
            type Target = $type;

            fn write(
                self,
                write: &mut (impl AsyncWriteExt + Unpin + Send),
            ) -> impl Future<Output = io::Result<()>> {
                async move {
                    write.write_u8(expression_discriminant::VALUE).await?;
                    Self::write_schema(write).await?;
                    self.write_value(write).await
                }
            }
        }
    )*};
}

// TODO: `Expression` should be implemented in #[derive(Schema)]
impl_expr!(
    () bool String
    u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
    f32 f64
    Duration
);

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
