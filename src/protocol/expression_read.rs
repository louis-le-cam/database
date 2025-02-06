use std::{io, marker::PhantomData};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    io_error,
    protocol::{expression_discriminant, FromStream, SchemaRead, ValueRead},
};

pub struct ExpressionRead<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>>(
    S,
    PhantomData<Next>,
);

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> FromStream<S>
    for ExpressionRead<S, Next>
{
    fn from_stream(stream: S) -> Self {
        Self(stream, PhantomData)
    }
}

pub enum ExpressionReadResult<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> {
    Path(Vec<u32>),
    Value(SchemaRead<S, ValueRead<S, Next>>),
    Set(ExpressionRead<S, ExpressionRead<S, Next>>),
    Equal(ExpressionRead<S, ExpressionRead<S, Next>>),
    Filter(ExpressionRead<S, ExpressionRead<S, Next>>),
    And(ExpressionRead<S, ExpressionRead<S, Next>>),
    MapVariant {
        discriminant: u32,
        operands: ExpressionRead<S, ExpressionRead<S, Next>>,
    },
    Chain(ExpressionRead<S, ExpressionRead<S, Next>>),
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin, Next: FromStream<S>> ExpressionRead<S, Next> {
    pub async fn read(mut self) -> io::Result<ExpressionReadResult<S, Next>> {
        Ok(match self.0.read_u8().await? {
            expression_discriminant::PATH => {
                let length = self.0.read_u32().await?;
                let mut path = Vec::with_capacity(length.try_into().map_err(|_| {
                    io_error!(
                        InvalidData,
                        "length of path expression doesn't fit into a pointer sized unsigned integer"
                    )
                })?);

                for _ in 0..length {
                    path.push(self.0.read_u32().await?);
                }

                ExpressionReadResult::Path(path)
            }
            expression_discriminant::VALUE => {
                ExpressionReadResult::Value(SchemaRead::from_stream(self.0))
            }
            expression_discriminant::SET => {
                ExpressionReadResult::Set(ExpressionRead::from_stream(self.0))
            }
            expression_discriminant::EQUAL => {
                ExpressionReadResult::Equal(ExpressionRead::from_stream(self.0))
            }
            expression_discriminant::FILTER => {
                ExpressionReadResult::Filter(ExpressionRead::from_stream(self.0))
            }
            expression_discriminant::AND => {
                ExpressionReadResult::And(ExpressionRead::from_stream(self.0))
            }
            expression_discriminant::MAP_VARIANT => ExpressionReadResult::MapVariant {
                discriminant: self.0.read_u32().await?,
                operands: ExpressionRead::from_stream(self.0),
            },
            expression_discriminant::CHAIN => {
                ExpressionReadResult::Chain(ExpressionRead::from_stream(self.0))
            }
            _ => {
                return Err(io_error!(
                    InvalidData,
                    "invalid discriminant for expression"
                ))
            }
        })
    }
}
