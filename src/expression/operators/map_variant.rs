use std::{future::Future, io, marker::PhantomData};

use crate::{expression_discriminant, Expression, Schema};

pub struct MapVariantExpression<L: Expression, R: Expression, Out: Schema + Send + Sync>(
    pub(crate) L,
    pub(crate) u32,
    pub(crate) R,
    pub(crate) PhantomData<Out>,
);

impl<L: Expression, R: Expression, Out: Schema + Send + Sync> Expression
    for MapVariantExpression<L, R, Out>
{
    type Target = Out;

    fn write(
        self,
        write: &mut (impl tokio::io::AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async move {
            write.write_u8(expression_discriminant::MAP_VARIANT).await?;
            Box::pin(self.0.write(write)).await?;
            write.write_u32(self.1).await?;
            Box::pin(self.2.write(write)).await?;
            Ok(())
        }
    }
}
