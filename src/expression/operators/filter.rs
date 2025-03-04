use std::collections::HashSet;

use crate::{Expression, FilterExpression, FromPath, Schema, Scope};

pub trait VecFilter<Rhs: Expression, T: Schema>: Expression + Sized {
    fn filter(self, filter: impl FnOnce(T::Expression) -> Rhs) -> FilterExpression<Self, Rhs>;
}

impl<Lhs: Expression<Target = Vec<T>>, Rhs: Expression<Target = bool>, T: Schema + Send + Sync>
    VecFilter<Rhs, T> for Lhs
{
    fn filter(self, filter: impl FnOnce(T::Expression) -> Rhs) -> FilterExpression<Self, Rhs> {
        Scope::increment_depth();
        let expression = (filter)(T::Expression::from_path(vec![Scope::get().unwrap()]));
        Scope::decrement_depth();

        FilterExpression(self, expression)
    }
}

pub trait HashSetFilter<Rhs: Expression, T: Schema>: Expression + Sized {
    fn filter(self, filter: impl FnOnce(T::Expression) -> Rhs) -> FilterExpression<Self, Rhs>;
}

impl<Lhs, Rhs, T> HashSetFilter<Rhs, T> for Lhs
where
    Lhs: Expression<Target = HashSet<T>>,
    Rhs: Expression<Target = bool>,
    T: Schema + Send + Sync,
{
    fn filter(self, filter: impl FnOnce(T::Expression) -> Rhs) -> FilterExpression<Self, Rhs> {
        Scope::increment_depth();
        let expression = (filter)(T::Expression::from_path(vec![Scope::get().unwrap()]));
        Scope::decrement_depth();

        FilterExpression(self, expression)
    }
}
