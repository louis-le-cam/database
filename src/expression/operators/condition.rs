use std::marker::PhantomData;

use crate::{ConditionExpression, Expression, Schema};

pub trait BoolOperators: Expression<Target = bool> + Sized {
    fn if_else<T: Schema, I: Expression<Target = T>, E: Expression<Target = T>>(
        self,
        if_branch: I,
        else_branch: E,
    ) -> ConditionExpression<Self, T, I, E>;
}

impl<B: Expression<Target = bool>> BoolOperators for B {
    fn if_else<T: Schema, I: Expression<Target = T>, E: Expression<Target = T>>(
        self,
        if_branch: I,
        else_branch: E,
    ) -> ConditionExpression<Self, T, I, E> {
        ConditionExpression(self, if_branch, else_branch, PhantomData)
    }
}
