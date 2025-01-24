use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{Schema, UnitExpression};

impl Schema for () {
    type Expression = UnitExpression;

    fn write_schema(
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        write.write_u8(5)
    }

    fn write_value(
        &self,
        _write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        std::future::ready(Ok(()))
    }

    fn read_value(
        _read: &mut (impl tokio::io::AsyncReadExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<Self>> + Send {
        std::future::ready(Ok(()))
    }
}
