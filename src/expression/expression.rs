use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::Schema;

pub trait Expression {
    type Target: Schema;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin),
    ) -> impl Future<Output = io::Result<()>>;
}
