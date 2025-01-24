use std::{future::Future, io};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{Expression, FromPath};

pub trait Schema: Sized {
    type Expression: Expression + FromPath;

    fn write_schema(
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send;

    fn write_value(
        &self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send;

    fn read_value(
        read: &mut (impl AsyncReadExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<Self>> + Send;
}
