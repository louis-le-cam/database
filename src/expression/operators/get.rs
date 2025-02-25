use std::{future::Future, io, marker::PhantomData};

use crate::{expression_discriminant, Expression, Schema};

pub struct GetExpression<L: Expression, R: Expression, Out: Schema + Send + Sync>(
    L,
    R,
    PhantomData<Out>,
);

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

// pub trait SlotMapGet<I: Expression<Target = K>, K: Key>: Expression + Sized {
//     type Item: Schema + Send + Sync;

//     fn get(self, index: I) -> GetExpression<Self, I, Option<Self::Item>>;
// }

// impl<
//         E: Expression<Target = SlotMap<K, T>>,
//         Ke: Expression<Target = K>,
//         K: Key,
//         T: Schema + Send + Sync,
//     > Get<Ke> for E
// {
//     type Out = T;

//     fn get(self, index: Ke) -> Self::Out {
//         todo!()
//     }
// }
