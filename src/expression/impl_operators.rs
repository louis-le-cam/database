use std::{future::Future, io, marker::PhantomData};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, Expression, Schema};

pub struct SetExpression<L: Expression, R: Expression>(pub(crate) L, pub(crate) R);
pub struct EqualExpression<L: Expression, R: Expression>(pub(crate) L, pub(crate) R);
pub struct FilterExpression<L: Expression, R: Expression>(pub(crate) L, pub(crate) R);
pub struct MapExpression<L: Expression, R: Expression, Out: Schema>(
    pub(crate) L,
    pub(crate) R,
    pub(crate) PhantomData<Out>,
);
pub struct LengthExpression<L: Expression>(pub(crate) L);
pub struct InsertExpression<L: Expression, I: Expression, R: Expression>(
    pub(crate) L,
    pub(crate) I,
    pub(crate) R,
);
pub struct AndExpression<L: Expression, R: Expression>(pub(crate) L, pub(crate) R);
pub struct MapVariantExpression<L: Expression, R: Expression, Out: Schema + Send + Sync>(
    pub(crate) L,
    pub(crate) u32,
    pub(crate) R,
    pub(crate) PhantomData<Out>,
);
pub struct FuseExpression<E: Expression, Out: Schema>(pub(crate) E, pub(crate) PhantomData<Out>);
pub struct ChainExpression<L: Expression, R: Expression>(pub(crate) L, pub(crate) R);
pub struct GetExpression<L: Expression, R: Expression, Out: Schema + Send + Sync>(
    pub(crate) L,
    pub(crate) R,
    pub(crate) PhantomData<Out>,
);
pub struct ConditionExpression<C, T, I, E>(
    pub(crate) C,
    pub(crate) I,
    pub(crate) E,
    pub(crate) PhantomData<T>,
)
where
    C: Expression<Target = bool>,
    T: Schema,
    I: Expression<Target = T>,
    E: Expression<Target = T>;

impl<L: Expression, R: Expression> Expression for SetExpression<L, R> {
    type Target = L::Target;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::SET).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Ok(())
        }
    }
}

impl<L: Expression, R: Expression> Expression for EqualExpression<L, R> {
    type Target = bool;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::EQUAL).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Ok(())
        }
    }
}

impl<L: Expression, R: Expression> Expression for FilterExpression<L, R>
where
    L::Target: Send + Sync,
{
    type Target = L::Target;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::FILTER).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Ok(())
        }
    }
}

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

impl<L: Expression> Expression for LengthExpression<L> {
    type Target = u32;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::LENGTH).await?;
            Box::pin(self.0.write(write)).await?;
            Ok(())
        }
    }
}

impl<L: Expression, I: Expression, R: Expression> Expression for InsertExpression<L, I, R> {
    type Target = L::Target;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::INSERT).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Box::pin(self.2.write(write)).await?;
            Ok(())
        }
    }
}

impl<L: Expression, R: Expression> Expression for AndExpression<L, R>
where
    L::Target: Send + Sync,
{
    type Target = L::Target;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::AND).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Ok(())
        }
    }
}

impl<L: Expression, R: Expression, Out: Schema + Send + Sync> Expression
    for MapVariantExpression<L, R, Out>
{
    type Target = Out;

    fn write(
        self,
        write: &mut (impl tokio::io::AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async move {
            write.write_u8(expression_discriminant::MAP_VARIANT).await?;
            Box::pin(self.0.write(write)).await?;
            write.write_u32(self.1).await?;
            Box::pin(self.2.write(write)).await?;
            Ok(())
        }
    }
}

impl<E: Expression, Out: Schema> Expression for FuseExpression<E, Out> {
    type Target = Out;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::FUSE).await?;
            Box::pin(self.0.write(write)).await?;
            Ok(())
        }
    }
}

impl<L: Expression, R: Expression> Expression for ChainExpression<L, R> {
    type Target = R::Target;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::CHAIN).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Ok(())
        }
    }
}

impl<L: Expression, R: Expression, Out: Schema + Send + Sync> Expression
    for GetExpression<L, R, Out>
{
    type Target = Out;

    fn write(
        self,
        write: &mut (impl tokio::io::AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::GET).await?;
            Box::pin(self.0.write(write)).await?;
            Box::pin(self.1.write(write)).await?;
            Ok(())
        }
    }
}

impl<C, T, I, E> Expression for ConditionExpression<C, T, I, E>
where
    C: Expression<Target = bool>,
    T: Schema,
    I: Expression<Target = T>,
    E: Expression<Target = T>,
{
    type Target = T;

    fn write(
        self,
        write: &mut (impl tokio::io::AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = std::io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::CONDITION).await?;
            self.0.write(write).await?;
            self.1.write(write).await?;
            self.2.write(write).await?;

            Ok(())
        }
    }
}
