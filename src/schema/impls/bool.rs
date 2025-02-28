use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{schema_discriminant, PathExpression, Schema};

impl Schema for bool {
    type Expression = PathExpression<bool>;

    fn write_schema(
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        write.write_u8(schema_discriminant::BOOLEAN)
    }

    fn write_value(
        &self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        write.write_u8(*self as u8)
    }

    fn read_value(
        read: &mut (impl tokio::io::AsyncReadExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<Self>> + Send {
        async { Ok(read.read_u8().await? != 0) }
    }
}
