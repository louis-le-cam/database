use std::{future::Future, io};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    expression_discriminant, io_error, schema_discriminant, Expression, PathExpression, Schema,
};

impl<S: Schema + Send + Sync> Schema for Option<S> {
    type Expression = PathExpression<Option<S>>;

    fn write_schema(
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        async {
            write.write_u8(schema_discriminant::SUM).await?;
            write.write_u32(2).await?;
            <()>::write_schema(write).await?;
            S::write_schema(write).await?;

            Ok(())
        }
    }

    fn write_value(
        &self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        async move {
            match self {
                None => write.write_u32(0).await?,
                Some(value) => {
                    write.write_u32(1).await?;
                    value.write_value(write).await?;
                }
            }

            Ok(())
        }
    }

    fn read_value(
        read: &mut (impl AsyncReadExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<Self>> + Send {
        async {
            match read.read_u32().await? {
                0 => Ok(None),
                1 => Ok(Some(S::read_value(read).await?)),
                _ => Err(io_error!(
                    InvalidData,
                    "invalid discriminant in value for a sum value"
                )),
            }
        }
    }
}

impl<T: Expression> Expression for Option<T>
where
    T::Target: Send + Sync,
{
    type Target = Option<T::Target>;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::SUM).await?;
            match self {
                None => {
                    write.write_u32(0).await?;
                    ().write(write).await?;
                }
                Some(expression) => {
                    write.write_u32(1).await?;
                    expression.write(write).await?;
                }
            }

            Ok(())
        }
    }
}

pub enum OptionMapped<Some, None> {
    Some(Some),
    None(None),
}

impl<Some: Schema + Send + Sync, None: Schema + Send + Sync> Schema for OptionMapped<Some, None> {
    type Expression = PathExpression<OptionMapped<Some, None>>;

    fn write_schema(
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        async {
            write.write_u8(schema_discriminant::PRODUCT).await?;
            write.write_u32(2).await?;
            None::write_schema(write).await?;
            Some::write_schema(write).await?;
            Ok(())
        }
    }

    fn write_value(
        &self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        async move {
            match self {
                OptionMapped::None(none) => {
                    write.write_u32(0).await?;
                    none.write_value(write).await?;
                }
                OptionMapped::Some(some) => {
                    write.write_u32(1).await?;
                    some.write_value(write).await?;
                }
            }

            Ok(())
        }
    }

    fn read_value(
        read: &mut (impl AsyncReadExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<Self>> + Send {
        async {
            match read.read_u32().await? {
                0 => Ok(OptionMapped::None(None::read_value(read).await?)),
                1 => Ok(OptionMapped::Some(Some::read_value(read).await?)),
                _ => Err(io_error!(
                    InvalidData,
                    "invalid discriminant in value for a sum value"
                )),
            }
        }
    }
}

impl<Some: Expression, None: Expression> Expression for OptionMapped<Some, None>
where
    Some::Target: Send + Sync,
    None::Target: Send + Sync,
{
    type Target = OptionMapped<Some::Target, None::Target>;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async {
            write.write_u8(expression_discriminant::SUM).await?;
            match self {
                Self::None(expression) => {
                    write.write_u32(0).await?;
                    expression.write(write).await?;
                }
                Self::Some(expression) => {
                    write.write_u32(1).await?;
                    expression.write(write).await?;
                }
            }

            Ok(())
        }
    }
}
