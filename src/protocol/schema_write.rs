use std::{io, marker::PhantomData};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::protocol::{schema_discriminant, FromStream};

pub struct SchemaWrite<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>>(
    S,
    PhantomData<Next>,
);

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> FromStream<S>
    for SchemaWrite<S, Next>
{
    fn from_stream(stream: S) -> Self {
        Self(stream, PhantomData)
    }
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> SchemaWrite<S, Next> {
    pub async fn product(mut self, count: u32) -> io::Result<SchemasWrite<S, Next>> {
        self.0.write_u8(schema_discriminant::PRODUCT).await?;
        self.0.write_u32(count).await?;
        Ok(SchemasWrite::from_stream(self.0))
    }

    pub async fn sum(mut self, count: u32) -> io::Result<SchemasWrite<S, Next>> {
        self.0.write_u8(schema_discriminant::SUM).await?;
        self.0.write_u32(count).await?;
        Ok(SchemasWrite::from_stream(self.0))
    }

    pub async fn list(mut self) -> io::Result<SchemaWrite<S, Next>> {
        self.0.write_u8(schema_discriminant::LIST).await?;
        Ok(SchemaWrite::from_stream(self.0))
    }

    pub async fn string(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::STRING).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn boolean(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::BOOLEAN).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn unit(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::UNIT).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn uint8(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::UINT8).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn uint16(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::UINT16).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn uint32(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::UINT32).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn uint64(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::UINT64).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn uint128(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::UINT128).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn int8(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::INT8).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn int16(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::INT16).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn int32(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::INT32).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn int64(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::INT64).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn int128(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::INT128).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn float32(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::FLOAT32).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn float64(mut self) -> io::Result<Next> {
        self.0.write_u8(schema_discriminant::FLOAT64).await?;
        Ok(Next::from_stream(self.0))
    }
}

pub struct SchemasWrite<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>>(
    S,
    PhantomData<Next>,
);

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> FromStream<S>
    for SchemasWrite<S, Next>
{
    fn from_stream(stream: S) -> Self {
        Self(stream, PhantomData)
    }
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> SchemasWrite<S, Next> {
    pub fn add(self) -> SchemaWrite<S, Self> {
        SchemaWrite::from_stream(self.0)
    }

    pub fn finish(self) -> Next {
        Next::from_stream(self.0)
    }
}
