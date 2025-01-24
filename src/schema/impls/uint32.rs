use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{BoolExpression, Schema};

impl Schema for u32 {
    type Expression = BoolExpression;

    fn write_schema(
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        write.write_u8(4)
    }

    fn write_value(
        &self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        write.write_u32(*self)
    }

    fn read_value(
        read: &mut (impl tokio::io::AsyncReadExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<Self>> + Send {
        read.read_u32()
    }
}
