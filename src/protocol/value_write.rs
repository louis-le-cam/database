use std::{io, marker::PhantomData};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::protocol::FromStream;

pub struct ValueWrite<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>>(
    S,
    PhantomData<Next>,
);

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> FromStream<S>
    for ValueWrite<S, Next>
{
    fn from_stream(stream: S) -> Self {
        Self(stream, PhantomData)
    }
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> ValueWrite<S, Next> {
    pub async fn product(mut self, length: u32) -> io::Result<ValuesWrite<S, Next>> {
        self.0.write_u32(length).await?;
        Ok(ValuesWrite::from_stream(self.0))
    }

    pub async fn sum(mut self, discriminant: u32) -> io::Result<ValueWrite<S, Next>> {
        self.0.write_u32(discriminant).await?;
        Ok(ValueWrite::from_stream(self.0))
    }

    pub async fn list(mut self, length: u32) -> io::Result<ValuesWrite<S, Next>> {
        self.0.write_u32(length).await?;
        Ok(ValuesWrite::from_stream(self.0))
    }

    pub async fn string(mut self, value: &str) -> io::Result<Next> {
        self.0.write(value.as_bytes()).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn boolean(mut self, value: bool) -> io::Result<Next> {
        self.0.write_u8(value as u8).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn unit(self) -> io::Result<Next> {
        Ok(Next::from_stream(self.0))
    }

    pub async fn uint8(mut self, value: u8) -> io::Result<Next> {
        self.0.write_u8(value).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn uint16(mut self, value: u16) -> io::Result<Next> {
        self.0.write_u16(value).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn uint32(mut self, value: u32) -> io::Result<Next> {
        self.0.write_u32(value).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn uint64(mut self, value: u64) -> io::Result<Next> {
        self.0.write_u64(value).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn uint128(mut self, value: u128) -> io::Result<Next> {
        self.0.write_u128(value).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn int8(mut self, value: i8) -> io::Result<Next> {
        self.0.write_i8(value).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn int16(mut self, value: i16) -> io::Result<Next> {
        self.0.write_i16(value).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn int32(mut self, value: i32) -> io::Result<Next> {
        self.0.write_i32(value).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn int64(mut self, value: i64) -> io::Result<Next> {
        self.0.write_i64(value).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn int128(mut self, value: i128) -> io::Result<Next> {
        self.0.write_i128(value).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn float32(mut self, value: f32) -> io::Result<Next> {
        self.0.write_f32(value).await?;
        Ok(Next::from_stream(self.0))
    }

    pub async fn float64(mut self, value: f64) -> io::Result<Next> {
        self.0.write_f64(value).await?;
        Ok(Next::from_stream(self.0))
    }
}

pub struct ValuesWrite<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>>(
    S,
    PhantomData<Next>,
);

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> FromStream<S>
    for ValuesWrite<S, Next>
{
    fn from_stream(stream: S) -> Self {
        Self(stream, PhantomData)
    }
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> ValuesWrite<S, Next> {
    pub fn add(self) -> ValueWrite<S, Self> {
        ValueWrite::from_stream(self.0)
    }

    pub fn finish(self) -> Next {
        Next::from_stream(self.0)
    }
}
