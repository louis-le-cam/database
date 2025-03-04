use crate::{Expression, InsertExpression};

pub trait VecInsert<I: Expression, R: Expression>: Expression + Sized {
    fn insert(self, index: I, value: R) -> InsertExpression<Self, I, R>;
}

impl<L, T, I, R> VecInsert<I, R> for L
where
    L: Expression<Target = Vec<T>>,
    T: Expression,
    I: Expression<Target = u32>,
    R: Expression<Target = T::Target>,
{
    fn insert(self, index: I, value: R) -> InsertExpression<Self, I, R> {
        InsertExpression(self, index, value)
    }
}
