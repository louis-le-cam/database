use std::{future::Future, io, marker::PhantomData};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, Expression, FromPath, Schema, Scope};

pub struct MapExpression<L: Expression, R: Expression, Out: Schema>(L, R, PhantomData<Out>);

impl<L: Expression, R: Expression, Out: Schema> Expression for MapExpression<L, R, Out>
where
    L::Target: Send + Sync,
{
    type Target = Out;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::MAP).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Ok(())
        }
    }
}

pub trait MapVec<Rhs: Expression, T: Schema>: Expression + Sized
where
    Rhs::Target: Send + Sync,
{
    fn map(
        self,
        map: impl FnOnce(T::Expression) -> Rhs,
    ) -> MapExpression<Self, Rhs, Vec<Rhs::Target>>;
}

impl<Lhs: Expression<Target = Vec<T>>, Rhs: Expression, T: Schema + Send + Sync> MapVec<Rhs, T>
    for Lhs
where
    Rhs::Target: Send + Sync,
{
    fn map(
        self,
        map: impl FnOnce(T::Expression) -> Rhs,
    ) -> MapExpression<Self, Rhs, Vec<Rhs::Target>> {
        Scope::increment_depth();
        let expression = (map)(T::Expression::from_path(vec![Scope::get().unwrap()]));
        Scope::decrement_depth();

        MapExpression(self, expression, PhantomData)
    }
}
