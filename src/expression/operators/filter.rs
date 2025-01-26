use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, Expression, FromPath, Schema, Scope};

pub struct FilterExpression<L: Expression, R: Expression>(L, R);

impl<L: Expression, R: Expression> Expression for FilterExpression<L, R>
where
    L::Target: Send + Sync,
{
    type Target = L::Target;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::FILTER).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Ok(())
        }
    }
}

pub trait Filter<Rhs: Expression, T: Schema>: Expression + Sized {
    fn filter(self, filter: impl FnOnce(T::Expression) -> Rhs) -> FilterExpression<Self, Rhs>;
}

impl<Lhs: Expression<Target = Vec<T>>, Rhs: Expression<Target = bool>, T: Schema + Send + Sync>
    Filter<Rhs, T> for Lhs
{
    fn filter(self, filter: impl FnOnce(T::Expression) -> Rhs) -> FilterExpression<Self, Rhs> {
        Scope::increment_depth();
        let expression = (filter)(T::Expression::from_path(vec![Scope::get().unwrap()]));
        Scope::decrement_depth();

        FilterExpression(self, expression)
    }
}
