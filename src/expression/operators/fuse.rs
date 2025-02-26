use std::{future::Future, io, marker::PhantomData};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, Expression, Schema};

pub struct FuseExpression<E: Expression, Out: Schema>(pub(crate) E, pub(crate) PhantomData<Out>);

impl<E: Expression, Out: Schema> Expression for FuseExpression<E, Out> {
    type Target = Out;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::FUSE).await?;
            Box::pin(self.0.write(write)).await?;
            Ok(())
        }
    }
}
