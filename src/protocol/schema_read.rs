use std::{io, marker::PhantomData};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    io_error,
    protocol::{schema_discriminant, FromStream},
};

pub struct SchemaRead<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>>(
    S,
    PhantomData<Next>,
);

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> FromStream<S>
    for SchemaRead<S, Next>
{
    fn from_stream(stream: S) -> Self {
        Self(stream, PhantomData)
    }
}

pub enum SchemaReadResult<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> {
    Product {
        length: u32,
        fields: SchemasRead<S, Next>,
    },
    Sum {
        length: u32,
        variants: SchemasRead<S, Next>,
    },
    List(SchemaRead<S, Next>),
    String(Next),
    Boolean(Next),
    Unit(Next),
    Uint8(Next),
    Uint16(Next),
    Uint32(Next),
    Uint64(Next),
    Uint128(Next),
    Int8(Next),
    Int16(Next),
    Int32(Next),
    Int64(Next),
    Int128(Next),
    Float32(Next),
    Float64(Next),
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> SchemaRead<S, Next> {
    pub async fn read(mut self) -> io::Result<SchemaReadResult<S, Next>> {
        Ok(match self.0.read_u8().await? {
            schema_discriminant::PRODUCT => SchemaReadResult::Product {
                length: self.0.read_u32().await?,
                fields: SchemasRead::from_stream(self.0),
            },
            schema_discriminant::SUM => SchemaReadResult::Sum {
                length: self.0.read_u32().await?,
                variants: SchemasRead::from_stream(self.0),
            },
            schema_discriminant::LIST => SchemaReadResult::List(SchemaRead::from_stream(self.0)),
            schema_discriminant::STRING => SchemaReadResult::String(Next::from_stream(self.0)),
            schema_discriminant::BOOLEAN => SchemaReadResult::Boolean(Next::from_stream(self.0)),
            schema_discriminant::UNIT => SchemaReadResult::Unit(Next::from_stream(self.0)),
            schema_discriminant::UINT8 => SchemaReadResult::Uint8(Next::from_stream(self.0)),
            schema_discriminant::UINT16 => SchemaReadResult::Uint16(Next::from_stream(self.0)),
            schema_discriminant::UINT32 => SchemaReadResult::Uint32(Next::from_stream(self.0)),
            schema_discriminant::UINT64 => SchemaReadResult::Uint64(Next::from_stream(self.0)),
            schema_discriminant::UINT128 => SchemaReadResult::Uint128(Next::from_stream(self.0)),
            schema_discriminant::INT8 => SchemaReadResult::Int8(Next::from_stream(self.0)),
            schema_discriminant::INT16 => SchemaReadResult::Int16(Next::from_stream(self.0)),
            schema_discriminant::INT32 => SchemaReadResult::Int32(Next::from_stream(self.0)),
            schema_discriminant::INT64 => SchemaReadResult::Int64(Next::from_stream(self.0)),
            schema_discriminant::INT128 => SchemaReadResult::Int128(Next::from_stream(self.0)),
            schema_discriminant::FLOAT32 => SchemaReadResult::Float32(Next::from_stream(self.0)),
            schema_discriminant::FLOAT64 => SchemaReadResult::Float64(Next::from_stream(self.0)),
            _ => return Err(io_error!(InvalidData, "invalid discriminant for schema")),
        })
    }
}

pub struct SchemasRead<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>>(
    S,
    PhantomData<Next>,
);

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> FromStream<S>
    for SchemasRead<S, Next>
{
    fn from_stream(stream: S) -> Self {
        Self(stream, PhantomData)
    }
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> SchemasRead<S, Next> {
    pub async fn next(self) -> io::Result<SchemaReadResult<S, Self>> {
        SchemaRead::from_stream(self.0).read().await
    }

    pub fn finish(self) -> Next {
        Next::from_stream(self.0)
    }
}
