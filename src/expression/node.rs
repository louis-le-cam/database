use std::{borrow::Cow, io};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{io_error, Value};

pub enum ExpressionNode {
    Path(Vec<u32>),
    Equal(Box<(ExpressionNode, ExpressionNode)>),
}

impl ExpressionNode {
    pub fn evaluate<'a>(self, value: &'a Value) -> Cow<'a, Value> {
        match self {
            ExpressionNode::Path(path) => Cow::Borrowed(value.scope(&path).unwrap()),
            ExpressionNode::Equal(operands) => {
                let (left_expression, right_expression) = *operands;

                let left_value = left_expression.evaluate(value);
                let right_value = right_expression.evaluate(value);

                Cow::Owned(Value::Boolean(left_value.equal(&right_value)))
            }
        }
    }

    fn discriminant(&self) -> u8 {
        match self {
            ExpressionNode::Path(_) => 0,
            ExpressionNode::Equal(_) => 1,
        }
    }

    pub async fn read(read: &mut (impl AsyncReadExt + Unpin)) -> io::Result<Self> {
        let discriminant = read.read_u8().await?;

        let node = match discriminant {
            0 => {
                let length: usize = read.read_u32().await?.try_into().map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "path expression length doesn't fit into a pointer sized unsigned integer",
                    )
                })?;

                let mut path = Vec::new();
                path.try_reserve(length).map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "allocation of memory for path expression failed"
                    )
                })?;

                for _ in 0..length {
                    path.push(read.read_u32().await?);
                }

                Self::Path(path)
            }
            1 => Self::Equal(Box::new((
                Box::pin(Self::read(read)).await?,
                Box::pin(Self::read(read)).await?,
            ))),
            _ => {
                return Err(io_error!(
                    InvalidData,
                    "invalid discriminant while parsing expression node",
                ));
            }
        };

        debug_assert_eq!(node.discriminant(), discriminant);

        Ok(node)
    }

    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        write.write_u8(self.discriminant()).await?;

        match self {
            ExpressionNode::Path(segments) => {
                write
                    .write_u32(segments.len().try_into().map_err(|_| {
                        io_error!(
                            OutOfMemory,
                            "path expression length doesn't fit into a 32 bit unsigned integer",
                        )
                    })?)
                    .await?;

                for segment in segments {
                    write.write_u32(*segment).await?;
                }
            }
            ExpressionNode::Equal(operands) => {
                Box::pin(operands.as_ref().0.write(write)).await?;
                Box::pin(operands.as_ref().1.write(write)).await?;
            }
        }

        Ok(())
    }
}
