use crate::{AndExpression, Expression};

pub trait And<Rhs: Expression>: Expression + Sized {
    fn and(self, filter: Rhs) -> AndExpression<Self, Rhs>;
}

impl<Lhs: Expression<Target = bool>, Rhs: Expression<Target = bool>> And<Rhs> for Lhs {
    fn and(self, rhs: Rhs) -> AndExpression<Self, Rhs> {
        AndExpression(self, rhs)
    }
}
