use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, Expression, FromPath, Schema, Scope};

pub struct EqualExpression<L: Expression, R: Expression>(L, R);

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

macro_rules! impl_equal {
    ($($trait:ident $target:ty;)*) => {
        $(
            pub trait $trait<Rhs: Expression>: Expression + Sized {
                fn equal(self, rhs: Rhs) -> EqualExpression<Self, Rhs>;
            }

            impl<L: Expression<Target = $target>, R: Expression<Target = $target>> $trait<R> for L {
                fn equal(self, rhs: R) -> EqualExpression<Self, R> {
                    EqualExpression(self, rhs)
                }
            }
        )*
    };
}

impl_equal!(
    StringEqual String;
    Uint8Equal u8;
    Uint16Equal u16;
    Uint32Equal u32;
    Uint64Equal u64;
    Uint128Equal u128;
    Int8Equal i8;
    Int16Equal i16;
    Int32Equal i32;
    Int64Equal i64;
    Int128Equal i128;
);

pub struct SetExpression<L: Expression, R: Expression>(L, R);

impl<L: Expression, R: Expression> Expression for SetExpression<L, R> {
    type Target = ();

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

pub trait Set<Rhs: Expression>: Expression + Sized {
    fn set(self, rhs: Rhs) -> SetExpression<Self, Rhs>;
}

impl<L: Expression<Target = T>, R: Expression<Target = T>, T> Set<R> for L {
    fn set(self, rhs: R) -> SetExpression<Self, R> {
        SetExpression(self, rhs)
    }
}

pub struct FilterExpression<L: Expression, R: Expression>(L, R);

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

pub trait Filter<Rhs: Expression, T: Schema>: Expression + Sized {
    fn filter(self, filter: impl FnOnce(T::Expression) -> Rhs) -> FilterExpression<Self, Rhs>;
}

impl<Lhs: Expression<Target = Vec<T>>, Rhs: Expression<Target = bool>, T: Schema + Send + Sync>
    Filter<Rhs, T> for Lhs
{
    fn filter(self, filter: impl FnOnce(T::Expression) -> Rhs) -> FilterExpression<Self, Rhs> {
        Scope::increment_depth();
        let expression = (filter)(T::Expression::from_path(vec![Scope::get().unwrap()]));
        Scope::decrement_depth();

        FilterExpression(self, expression)
    }
}
