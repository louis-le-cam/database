use std::marker::PhantomData;

use crate::{Expression, FromPath, MapExpression, Schema, Scope};

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
