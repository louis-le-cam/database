use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, Expression};

pub struct LengthExpression<L: Expression>(L);

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

pub trait Length: Expression + Sized {
    fn length(self) -> LengthExpression<Self>;
}

impl<Lhs: Expression<Target = Vec<T>>, T: Expression> Length for Lhs {
    fn length(self) -> LengthExpression<Self> {
        LengthExpression(self)
    }
}
