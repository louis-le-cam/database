use std::marker::PhantomData;

use crate::{Expression, GetExpression, Schema};

pub trait VecGet<I: Expression<Target = u32>>: Expression + Sized {
    type Item: Schema + Send + Sync;

    fn get(self, index: I) -> GetExpression<Self, I, Option<Self::Item>>;
}

impl<E: Expression<Target = Vec<T>>, T: Schema + Send + Sync, I: Expression<Target = u32>> VecGet<I>
    for E
{
    type Item = T;

    fn get(self, index: I) -> GetExpression<Self, I, Option<T>> {
        GetExpression(self, index, PhantomData)
    }
}
