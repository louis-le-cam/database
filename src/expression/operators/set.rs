use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, Chain, Expression, OptionOperators, Schema};

pub struct SetExpression<L: Expression, R: Expression>(L, R);

impl<L: Expression, R: Expression> Expression for SetExpression<L, R> {
    type Target = L::Target;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::SET).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Ok(())
        }
    }
}

pub trait Set<Rhs: Expression>: Expression + Sized {
    fn set(self, rhs: Rhs) -> SetExpression<Self, Rhs>;
}

impl<L: Expression<Target = T>, R: Expression<Target = T>, T> Set<R> for L {
    fn set(self, rhs: R) -> SetExpression<Self, R> {
        SetExpression(self, rhs)
    }
}

pub trait SetIfSome<Rhs: Expression>: Expression + Sized {
    fn set_if_some(self, rhs: Rhs) -> impl Expression<Target = ()>;
}

impl<
        L: Expression<Target = T>,
        R: Expression<Target = Option<T>>,
        T: Schema<Expression = Te> + Send + Sync,
        Te: Expression<Target = T>,
    > SetIfSome<R> for L
{
    fn set_if_some(self, rhs: R) -> impl Expression<Target = ()> {
        OptionOperators::map(rhs, |rhs| self.set(rhs).chain(())).unwrap_or(())
    }
}
