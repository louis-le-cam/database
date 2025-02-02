use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, Expression};

pub struct ChainExpression<L: Expression, R: Expression>(L, R);

impl<L: Expression, R: Expression> Expression for ChainExpression<L, R> {
    type Target = R::Target;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::CHAIN).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Ok(())
        }
    }
}

pub trait Chain<Rhs: Expression>: Expression + Sized {
    fn chain(self, rhs: Rhs) -> ChainExpression<Self, Rhs>;
}

impl<L: Expression, R: Expression> Chain<R> for L {
    fn chain(self, rhs: R) -> ChainExpression<Self, R> {
        ChainExpression(self, rhs)
    }
}
