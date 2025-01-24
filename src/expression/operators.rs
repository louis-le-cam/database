use std::{future::Future, io};

use super::Expression;

pub struct EqualExpression<L: Expression, R: Expression>(L, R);

impl<L: Expression, R: Expression> Expression for EqualExpression<L, R> {
    type Target = bool;

    fn write(
        self,
        write: &mut (impl tokio::io::AsyncWriteExt + Unpin),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(1).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Ok(())
        }
    }
}

pub trait Equal<Rhs: Expression>: Expression + Sized {
    fn equal(self, rhs: Rhs) -> EqualExpression<Self, Rhs>;
}

impl<L: Expression<Target = String>, R: Expression<Target = String>> Equal<R> for L {
    fn equal(self, rhs: R) -> EqualExpression<Self, R> {
        EqualExpression(self, rhs)
    }
}
