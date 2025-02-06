use std::{io, marker::PhantomData};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    io_error,
    protocol::{expression_discriminant, FromStream, SchemaWrite, ValueWrite},
};

pub struct ExpressionWrite<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>>(
    S,
    PhantomData<Next>,
);

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> FromStream<S>
    for ExpressionWrite<S, Next>
{
    fn from_stream(stream: S) -> Self {
        Self(stream, PhantomData)
    }
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> ExpressionWrite<S, Next> {
    pub async fn path(mut self, path: &[u32]) -> io::Result<Next> {
        self.0.write_u8(expression_discriminant::PATH).await?;
        self.0
            .write_u32(path.len().try_into().map_err(|_| {
                io_error!(
                    InvalidData,
                    "length of path expression doesn't fit in a 32 bit unsigned integer"
                )
            })?)
            .await?;

        for segment in path {
            self.0.write_u32(*segment).await?;
        }

        Ok(Next::from_stream(self.0))
    }

    pub async fn value(mut self) -> io::Result<SchemaWrite<S, ValueWrite<S, Next>>> {
        self.0.write_u8(expression_discriminant::VALUE).await?;
        Ok(SchemaWrite::from_stream(self.0))
    }

    pub async fn set(mut self) -> io::Result<ExpressionWrite<S, ExpressionWrite<S, Next>>> {
        self.0.write_u8(expression_discriminant::SET).await?;
        Ok(ExpressionWrite::from_stream(self.0))
    }

    pub async fn equal(mut self) -> io::Result<ExpressionWrite<S, ExpressionWrite<S, Next>>> {
        self.0.write_u8(expression_discriminant::EQUAL).await?;
        Ok(ExpressionWrite::from_stream(self.0))
    }

    pub async fn filter(mut self) -> io::Result<ExpressionWrite<S, ExpressionWrite<S, Next>>> {
        self.0.write_u8(expression_discriminant::FILTER).await?;
        Ok(ExpressionWrite::from_stream(self.0))
    }

    pub async fn and(mut self) -> io::Result<ExpressionWrite<S, ExpressionWrite<S, Next>>> {
        self.0.write_u8(expression_discriminant::AND).await?;
        Ok(ExpressionWrite::from_stream(self.0))
    }

    pub async fn map_variant(
        mut self,
        discriminant: u32,
    ) -> io::Result<ExpressionWrite<S, ExpressionWrite<S, Next>>> {
        self.0
            .write_u8(expression_discriminant::MAP_VARIANT)
            .await?;
        self.0.write_u32(discriminant).await?;
        Ok(ExpressionWrite::from_stream(self.0))
    }

    pub async fn chain(mut self) -> io::Result<ExpressionWrite<S, ExpressionWrite<S, Next>>> {
        self.0.write_u8(expression_discriminant::CHAIN).await?;
        Ok(ExpressionWrite::from_stream(self.0))
    }
}
