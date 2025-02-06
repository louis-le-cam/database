use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::protocol::FromStream;

pub struct RequestRead<S: AsyncWriteExt + AsyncReadExt + Unpin>(S);

impl<S: AsyncWriteExt + AsyncReadExt + Unpin> FromStream<S> for RequestRead<S> {
    fn from_stream(stream: S) -> Self {
        Self(stream)
    }
}
