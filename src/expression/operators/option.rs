use std::marker::PhantomData;

use crate::{
    Expression, FromPath, FuseExpression, MapVariantExpression, OptionMapped, Schema, Scope,
};

pub trait OptionOperators<T: Schema + Send + Sync>: Expression + Sized {
    fn map<N: Expression>(
        self,
        map: impl FnOnce(T::Expression) -> N,
    ) -> MapVariantExpression<Self, N, Option<N::Target>>
    where
        N::Target: Send + Sync;

    fn unwrap_or<E: Expression<Target = T> + Send + Sync>(
        self,
        default: E,
    ) -> FuseExpression<MapVariantExpression<Self, E, OptionMapped<T, E::Target>>, T>
    where
        MapVariantExpression<Self, E, OptionMapped<T, E::Target>>: Expression;
}

impl<S: Expression<Target = Option<T>>, T: Schema + Send + Sync> OptionOperators<T> for S {
    fn map<N: Expression>(
        self,
        map: impl FnOnce(T::Expression) -> N,
    ) -> MapVariantExpression<Self, N, Option<N::Target>>
    where
        N::Target: Send + Sync,
    {
        Scope::increment_depth();
        let expression = (map)(T::Expression::from_path(vec![Scope::get().unwrap()]));
        Scope::decrement_depth();

        MapVariantExpression(self, 1, expression, PhantomData)
    }

    fn unwrap_or<E: Expression<Target = T> + Send + Sync>(
        self,
        default: E,
    ) -> FuseExpression<MapVariantExpression<Self, E, OptionMapped<T, E::Target>>, T>
    where
        MapVariantExpression<Self, E, OptionMapped<T, E::Target>>: Expression,
    {
        FuseExpression(
            MapVariantExpression(self, 0, default, PhantomData),
            PhantomData,
        )
    }
}
