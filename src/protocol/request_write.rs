use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::protocol::{
    request_discriminant, ExpressionWrite, FromStream, SchemaRead, SchemaWrite, ValueRead,
    ValueWrite,
};

pub struct RequestWrite<S: AsyncWriteExt + AsyncReadExt + Unpin>(S);

impl<S: AsyncWriteExt + AsyncReadExt + Unpin> FromStream<S> for RequestWrite<S> {
    fn from_stream(stream: S) -> Self {
        Self(stream)
    }
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin> RequestWrite<S> {
    pub async fn get_schema(mut self) -> io::Result<SchemaRead<S, Self>> {
        self.0.write_u8(request_discriminant::GET_SCHEMA).await?;
        Ok(SchemaRead::from_stream(self.0))
    }

    pub async fn set_schema(
        mut self,
    ) -> io::Result<SchemaWrite<S, ValueWrite<S, RequestWrite<S>>>> {
        self.0.write_u8(request_discriminant::SET_SCHEMA).await?;
        Ok(SchemaWrite::from_stream(self.0))
    }

    pub async fn query(mut self) -> io::Result<ExpressionWrite<S, ValueRead<S, Self>>> {
        self.0.write_u8(request_discriminant::QUERY).await?;
        Ok(ExpressionWrite::from_stream(self.0))
    }
}
