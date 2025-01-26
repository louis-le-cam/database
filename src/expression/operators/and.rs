use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, Expression};

pub struct AndExpression<L: Expression, R: Expression>(L, R);

impl<L: Expression, R: Expression> Expression for AndExpression<L, R>
where
    L::Target: Send + Sync,
{
    type Target = L::Target;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::AND).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Ok(())
        }
    }
}

pub trait And<Rhs: Expression>: Expression + Sized {
    fn and(self, filter: Rhs) -> AndExpression<Self, Rhs>;
}

impl<Lhs: Expression<Target = bool>, Rhs: Expression<Target = bool>> And<Rhs> for Lhs {
    fn and(self, rhs: Rhs) -> AndExpression<Self, Rhs> {
        AndExpression(self, rhs)
    }
}
