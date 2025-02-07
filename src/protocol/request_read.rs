use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    io_error,
    protocol::{
        request_discriminant, ExpressionRead, FromStream, RequestWrite, SchemaRead, ValueRead,
    },
};

pub struct RequestRead<S: AsyncWriteExt + AsyncReadExt + Unpin>(S);

impl<S: AsyncWriteExt + AsyncReadExt + Unpin> FromStream<S> for RequestRead<S> {
    fn from_stream(stream: S) -> Self {
        Self(stream)
    }
}

pub enum RequestReadResult<S: AsyncWriteExt + AsyncReadExt + Unpin> {
    GetSchema(RequestWrite<S>),
    SetSchema(SchemaRead<S, ValueRead<S, RequestWrite<S>>>),
    Query(ExpressionRead<S, RequestWrite<S>>),
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin> RequestRead<S> {
    pub async fn read(mut self) -> io::Result<RequestReadResult<S>> {
        Ok(match self.0.read_u8().await? {
            request_discriminant::GET_SCHEMA => {
                RequestReadResult::GetSchema(RequestWrite::from_stream(self.0))
            }
            request_discriminant::SET_SCHEMA => {
                RequestReadResult::SetSchema(SchemaRead::from_stream(self.0))
            }
            request_discriminant::QUERY => {
                RequestReadResult::Query(ExpressionRead::from_stream(self.0))
            }
            _ => return Err(io_error!(InvalidData, "invalid discriminant for request")),
        })
    }
}
