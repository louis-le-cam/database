use std::{
    io,
    sync::{Arc, Mutex},
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{io_error, Value};

pub enum ExpressionNode {
    Path(Vec<u32>),
    Set(Box<(ExpressionNode, ExpressionNode)>),
    Equal(Box<(ExpressionNode, ExpressionNode)>),
}

pub mod expression_discriminant {
    pub const PATH: u8 = 0;
    pub const SET: u8 = 1;
    pub const EQUAL: u8 = 2;
}

impl ExpressionNode {
    pub fn evaluate(self, value: Arc<Mutex<Value>>) -> Arc<Mutex<Value>> {
        match self {
            ExpressionNode::Path(path) => Value::scope(value, &path).unwrap(),
            ExpressionNode::Set(operands) => {
                let (left_expression, right_expression) = *operands;

                *left_expression.evaluate(value.clone()).lock().unwrap() = right_expression
                    .evaluate(value.clone())
                    .lock()
                    .unwrap()
                    .clone();

                Arc::new(Mutex::new(Value::Unit))
            }
            ExpressionNode::Equal(operands) => {
                let (left_expression, right_expression) = *operands;

                let left_value = left_expression.evaluate(value.clone());
                let right_value = right_expression.evaluate(value);

                Arc::new(Mutex::new(Value::Boolean(
                    left_value
                        .clone()
                        .lock()
                        .unwrap()
                        .equal(&right_value.clone().lock().unwrap()),
                )))
            }
        }
    }

    fn discriminant(&self) -> u8 {
        match self {
            ExpressionNode::Path(_) => expression_discriminant::PATH,
            ExpressionNode::Set(_) => expression_discriminant::SET,
            ExpressionNode::Equal(_) => expression_discriminant::EQUAL,
        }
    }

    pub async fn read(read: &mut (impl AsyncReadExt + Unpin)) -> io::Result<Self> {
        let discriminant = read.read_u8().await?;

        let node = match discriminant {
            expression_discriminant::PATH => {
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
            expression_discriminant::SET => Self::Set(Box::new((
                Box::pin(Self::read(read)).await?,
                Box::pin(Self::read(read)).await?,
            ))),
            expression_discriminant::EQUAL => Self::Equal(Box::new((
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
            ExpressionNode::Set(operands) => {
                Box::pin(operands.as_ref().0.write(write)).await?;
                Box::pin(operands.as_ref().1.write(write)).await?;
            }
            ExpressionNode::Equal(operands) => {
                Box::pin(operands.as_ref().0.write(write)).await?;
                Box::pin(operands.as_ref().1.write(write)).await?;
            }
        }

        Ok(())
    }
}
