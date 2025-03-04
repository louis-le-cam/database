use crate::{Chain, Expression, OptionOperators, Schema, SetExpression};

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
