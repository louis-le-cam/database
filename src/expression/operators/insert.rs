use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, Expression};

pub struct InsertExpression<L: Expression, I: Expression, R: Expression>(L, I, R);

impl<L: Expression, I: Expression, R: Expression> Expression for InsertExpression<L, I, R> {
    type Target = L::Target;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::INSERT).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Box::pin(self.2.write(write)).await?;
            Ok(())
        }
    }
}

pub trait VecInsert<I: Expression, R: Expression>: Expression + Sized {
    fn insert(self, index: I, value: R) -> InsertExpression<Self, I, R>;
}

impl<L, T, I, R> VecInsert<I, R> for L
where
    L: Expression<Target = Vec<T>>,
    T: Expression,
    I: Expression<Target = u32>,
    R: Expression<Target = T::Target>,
{
    fn insert(self, index: I, value: R) -> InsertExpression<Self, I, R> {
        InsertExpression(self, index, value)
    }
}
