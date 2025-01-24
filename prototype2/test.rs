use std::{io, marker::PhantomData};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub enum SchemaNode {
    Product(Vec<SchemaNode>),
    Sum(Vec<SchemaNode>),
    List(Box<SchemaNode>),
    Leaf(SchemaLeaf),
}

#[derive(Clone, Copy)]
pub enum SchemaLeaf {
    String,
    U32,
}

impl SchemaNode {
    fn discriminant(&self) -> u8 {
        match self {
            Self::Product(_) => 0,
            Self::Sum(_) => 1,
            Self::List(_) => 2,
            Self::Leaf(SchemaLeaf::String) => 3,
            Self::Leaf(SchemaLeaf::U32) => 4,
        }
    }

    pub async fn read(read: &mut (impl AsyncReadExt + Unpin)) -> io::Result<Self> {
        let discriminant = read.read_u8().await?;

        let node = match discriminant {
            0 => {
                let length = read.read_u32().await? as usize;
                let mut fields = Vec::with_capacity(length);

                for _ in 0..length {
                    fields.push(Box::pin(Self::read(read)).await?);
                }

                Self::Product(fields)
            }
            1 => {
                let length = read.read_u32().await? as usize;
                let mut variants = Vec::with_capacity(length);

                for _ in 0..length {
                    variants.push(Box::pin(Self::read(read)).await?);
                }

                Self::Sum(variants)
            }
            2 => Self::List(Box::new(Box::pin(Self::read(read)).await?)),
            3 => Self::Leaf(SchemaLeaf::String),
            4 => Self::Leaf(SchemaLeaf::U32),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid discriminant while parsing schema node",
                ))
            }
        };

        debug_assert_eq!(node.discriminant(), discriminant);

        Ok(node)
    }

    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        write.write_u8(self.discriminant()).await?;

        match self {
            SchemaNode::Product(fields) => {
                write
                    .write_u32(fields.len().try_into().map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "schema product length doesn't fit in a 32 bit unsigned integer",
                        )
                    })?)
                    .await?;

                for field in fields {
                    Box::pin(field.write(write)).await?;
                }
            }
            SchemaNode::Sum(variants) => {
                write
                    .write_u32(variants.len().try_into().map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "schema sum length doesn't fit in a 32 bit unsigned integer",
                        )
                    })?)
                    .await?;

                for variant in variants {
                    Box::pin(variant.write(write)).await?;
                }
            }
            SchemaNode::List(inner) => Box::pin(inner.write(write)).await?,
            SchemaNode::Leaf(_) => {}
        }

        Ok(())
    }
}

pub trait Schema: Sized {
    type Expression: Expression + FromPath;

    async fn write_schema(write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()>;

    async fn write_value(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()>;

    async fn read_value(read: &mut (impl AsyncReadExt + Unpin)) -> io::Result<Self>;
}

impl<T: Schema> Schema for Vec<T> {
    type Expression = VecExpression<T>;

    async fn write_schema(write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        write.write_u8(2).await?;
        T::write_schema(write).await?;
        Ok(())
    }

    async fn write_value(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        write
            .write_u32(self.len().try_into().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "list value length doesn't fit in a 32 bit unsigned integer",
                )
            })?)
            .await?;

        for value in self {
            value.write_value(write).await?;
        }

        Ok(())
    }

    async fn read_value(read: &mut (impl AsyncReadExt + Unpin)) -> io::Result<Self> {
        let length = read.read_u32().await? as usize;

        let mut value = Vec::with_capacity(length);

        for _ in 0..length {
            value.push(T::read_value(read).await?);
        }

        Ok(value)
    }
}

pub struct VecExpression<T>(Vec<u32>, PhantomData<T>);
impl<T> FromPath for VecExpression<T> {
    fn from_path(path: Vec<u32>) -> Self {
        Self(path, PhantomData)
    }
}
impl<T: Schema> Expression for VecExpression<T> {
    type Target = Vec<T>;

    async fn write(self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        todo!()
    }
}

pub trait FromPath {
    fn from_path(path: Vec<u32>) -> Self;
}

pub trait Expression {
    type Target: Schema;

    async fn write(self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()>;
}

enum ExpressionNode {
    Path(Vec<u32>),
}

impl ExpressionNode {
    pub async fn read(read: &mut (impl AsyncReadExt + Unpin)) -> io::Result<Self> {
        match read.read_u8().await? {
            0 => {
                let length = read.read_u32().await? as usize;
                let mut path = Vec::with_capacity(length);

                for _ in 0..length {
                    path.push(read.read_u32().await?);
                }

                Ok(Self::Path(path))
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid discriminant while parsing expression",
                ))
            }
        }
    }
}
