use std::{future::Future, marker::PhantomData};

use crate::{expression_discriminant, Expression, Schema};

pub struct ConditionExpression<
    C: Expression<Target = bool>,
    T: Schema,
    I: Expression<Target = T>,
    E: Expression<Target = T>,
>(C, I, E, PhantomData<T>);

impl<
        C: Expression<Target = bool>,
        T: Schema,
        I: Expression<Target = T>,
        E: Expression<Target = T>,
    > Expression for ConditionExpression<C, T, I, E>
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
