use std::{io, marker::PhantomData};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{io_error, protocol::FromStream};

pub struct ValueRead<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>>(
    S,
    PhantomData<Next>,
);

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> FromStream<S>
    for ValueRead<S, Next>
{
    fn from_stream(stream: S) -> Self {
        Self(stream, PhantomData)
    }
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> ValueRead<S, Next> {
    pub async fn product(self) -> io::Result<ValuesRead<S, Next>> {
        Ok(ValuesRead::from_stream(self.0))
    }

    pub async fn sum(mut self) -> io::Result<(u32, ValueRead<S, Next>)> {
        Ok((self.0.read_u32().await?, ValueRead::from_stream(self.0)))
    }

    pub async fn list(mut self) -> io::Result<(u32, ValuesRead<S, Next>)> {
        Ok((self.0.read_u32().await?, ValuesRead::from_stream(self.0)))
    }

    pub async fn string(mut self) -> io::Result<(String, Next)> {
        let length = self.0.read_u32().await?;
        let mut buffer = vec![
            0;
            length.try_into().map_err(|_| io_error!(
                InvalidData,
                "length of string value doesn't fit into a pointer sized unsigned integer"
            ))?
        ];
        self.0.read_exact(&mut buffer).await?;
        Ok((
            String::from_utf8(buffer)
                .map_err(|_| io_error!(InvalidData, "string value is not valid utf-8"))?,
            Next::from_stream(self.0),
        ))
    }

    pub async fn boolean(mut self) -> io::Result<(bool, Next)> {
        Ok((self.0.read_u8().await? != 0, Next::from_stream(self.0)))
    }

    pub async fn unit(self) -> io::Result<Next> {
        Ok(Next::from_stream(self.0))
    }

    pub async fn uint8(mut self) -> io::Result<(u8, Next)> {
        Ok((self.0.read_u8().await?, Next::from_stream(self.0)))
    }

    pub async fn uint16(mut self) -> io::Result<(u16, Next)> {
        Ok((self.0.read_u16().await?, Next::from_stream(self.0)))
    }

    pub async fn uint32(mut self) -> io::Result<(u32, Next)> {
        Ok((self.0.read_u32().await?, Next::from_stream(self.0)))
    }

    pub async fn uint64(mut self) -> io::Result<(u64, Next)> {
        Ok((self.0.read_u64().await?, Next::from_stream(self.0)))
    }

    pub async fn uint128(mut self) -> io::Result<(u128, Next)> {
        Ok((self.0.read_u128().await?, Next::from_stream(self.0)))
    }

    pub async fn int8(mut self) -> io::Result<(i8, Next)> {
        Ok((self.0.read_i8().await?, Next::from_stream(self.0)))
    }

    pub async fn int16(mut self) -> io::Result<(i16, Next)> {
        Ok((self.0.read_i16().await?, Next::from_stream(self.0)))
    }

    pub async fn int32(mut self) -> io::Result<(i32, Next)> {
        Ok((self.0.read_i32().await?, Next::from_stream(self.0)))
    }

    pub async fn int64(mut self) -> io::Result<(i64, Next)> {
        Ok((self.0.read_i64().await?, Next::from_stream(self.0)))
    }

    pub async fn int128(mut self) -> io::Result<(i128, Next)> {
        Ok((self.0.read_i128().await?, Next::from_stream(self.0)))
    }

    pub async fn float32(mut self) -> io::Result<(f32, Next)> {
        Ok((self.0.read_f32().await?, Next::from_stream(self.0)))
    }

    pub async fn float64(mut self) -> io::Result<(f64, Next)> {
        Ok((self.0.read_f64().await?, Next::from_stream(self.0)))
    }
}

pub struct ValuesRead<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>>(
    S,
    PhantomData<Next>,
);

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> FromStream<S>
    for ValuesRead<S, Next>
{
    fn from_stream(stream: S) -> Self {
        Self(stream, PhantomData)
    }
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> ValuesRead<S, Next> {
    pub fn next(self) -> ValueRead<S, Self> {
        ValueRead::from_stream(self.0)
    }

    pub fn finish(self) -> Next {
        Next::from_stream(self.0)
    }
}
