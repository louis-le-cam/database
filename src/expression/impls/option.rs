use std::{io, marker::PhantomData};

use tokio::io::AsyncWriteExt;

use crate::{
    expression_discriminant, io_error, Expression, FromPath, FuseExpression, MapVariantExpression,
    OptionMapped, Schema, Scope,
};

pub struct OptionExpression<S: Schema + Send + Sync>(Vec<u32>, PhantomData<S>);

impl<S: Schema + Send + Sync> Clone for OptionExpression<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<S: Schema + Send + Sync> Expression for OptionExpression<S> {
    type Target = Option<S>;

    async fn write(self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        write.write_u8(expression_discriminant::PATH).await?;
        write
            .write_u32(self.0.len().try_into().map_err(|_| {
                io_error!(
                    OutOfMemory,
                    "path expression length doesn't fit into a 32 bit unsigned integer",
                )
            })?)
            .await?;

        for segment in &self.0 {
            write.write_u32(*segment).await?;
        }

        Ok(())
    }
}

impl<S: Schema + Send + Sync> FromPath for OptionExpression<S> {
    fn from_path(path: Vec<u32>) -> Self {
        Self(path, PhantomData)
    }
}

pub struct OptionMappedExpression<Some, None>(Vec<u32>, PhantomData<(Some, None)>);

impl<Some, None> FromPath for OptionMappedExpression<Some, None> {
    fn from_path(path: Vec<u32>) -> Self {
        Self(path, PhantomData)
    }
}

impl<Some: Schema + Send + Sync, None: Schema + Send + Sync> Expression
    for OptionMappedExpression<Some, None>
{
    type Target = OptionMapped<Some, None>;

    async fn write(self, write: &mut (impl AsyncWriteExt + Unpin + Send)) -> io::Result<()> {
        write.write_u8(expression_discriminant::PATH).await?;
        write
            .write_u32(self.0.len().try_into().map_err(|_| {
                io_error!(
                    OutOfMemory,
                    "path expression length doesn't fit into a 32 bit unsigned integer",
                )
            })?)
            .await?;

        for segment in &self.0 {
            write.write_u32(*segment).await?;
        }

        Ok(())
    }
}

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
