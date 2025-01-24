use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{io_error, SchemaLeaf, SchemaNode};

#[derive(Clone)]
pub enum Value {
    Product(Vec<Value>),
    Sum(u32, Box<Value>),
    List(Vec<Value>),
    Leaf(ValueLeaf),
}

#[derive(Clone)]
pub enum ValueLeaf {
    String(String),
    Uint32(u32),
    Boolean(bool),
}

impl Value {
    pub fn scope(&self, path: &[u32]) -> Option<&Self> {
        let Some((segment, segments)) = path.split_first() else {
            return Some(self);
        };

        match self {
            Value::Product(fields) => fields
                .get(usize::try_from(*segment).ok()?)
                .and_then(|value| value.scope(segments)),
            Value::Sum(discriminant, value) => (discriminant == segment)
                .then(|| value.scope(segments))
                .flatten(),
            Value::List(list) => list
                .get(usize::try_from(*segment).ok()?)
                .and_then(|value| value.scope(segments)),
            Value::Leaf(_) => None,
        }
    }

    pub fn equal(&self, rhs: &Value) -> bool {
        match (self, rhs) {
            (Value::Product(lhs), Value::Product(rhs)) => {
                debug_assert_eq!(lhs.len(), rhs.len());

                lhs.iter().zip(rhs).all(|(lhs, rhs)| lhs.equal(rhs))
            }
            (Value::Sum(lhs_discriminant, lhs), Value::Sum(rhs_discriminant, rhs)) => {
                (lhs_discriminant == rhs_discriminant) && lhs.equal(rhs)
            }
            (Value::List(lhs), Value::List(rhs)) => {
                lhs.len() == rhs.len() && lhs.iter().zip(rhs).all(|(lhs, rhs)| lhs.equal(rhs))
            }
            (Value::Leaf(ValueLeaf::String(lhs)), Value::Leaf(ValueLeaf::String(rhs))) => {
                lhs == rhs
            }
            (Value::Leaf(ValueLeaf::Uint32(lhs)), Value::Leaf(ValueLeaf::Uint32(rhs))) => {
                lhs == rhs
            }
            (Value::Leaf(ValueLeaf::Boolean(lhs)), Value::Leaf(ValueLeaf::Boolean(rhs))) => {
                lhs == rhs
            }
            _ => panic!(),
        }
    }

    pub async fn read(
        schema: &SchemaNode,
        read: &mut (impl AsyncReadExt + Unpin),
    ) -> io::Result<Self> {
        Ok(match schema {
            SchemaNode::Product(fields) => {
                let mut values = Vec::new();
                values.try_reserve(fields.len()).map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "allocation of memory for product values failed"
                    )
                })?;

                for field in fields {
                    values.push(Box::pin(Value::read(field, read)).await?);
                }

                Value::Product(values)
            }
            SchemaNode::Sum(variants) => {
                let discriminant = read.read_u32().await?;

                let variant = variants
                    .get(usize::try_from(discriminant).map_err(|_| {
                        io_error!(
                            InvalidData,
                            "discriminant in value for a sum schema doesn't fit into a pointer sized unsigned integer"
                        )
                    })?)
                    .ok_or(io_error!(
                        InvalidData,
                        "invalid discriminant in value for a sum schema"
                    ))?;

                Value::Sum(
                    discriminant,
                    Box::new(Box::pin(Value::read(variant, read)).await?),
                )
            }
            SchemaNode::List(inner) => {
                let length: usize = read.read_u32().await?.try_into().map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "list value length doesn't fit into a pointer sized unsigned integer",
                    )
                })?;

                let mut values = Vec::new();
                values.try_reserve(length).map_err(|_| {
                    io_error!(OutOfMemory, "allocation of memory for list value failed")
                })?;

                for _ in 0..length {
                    values.push(Box::pin(Value::read(inner, read)).await?);
                }

                Value::List(values)
            }
            SchemaNode::Leaf(leaf) => Value::Leaf(match leaf {
                SchemaLeaf::String => {
                    let length: usize = read.read_u32().await?.try_into().map_err(|_| {
                        io_error!(
                            OutOfMemory,
                            "string value length doesn't fit into a pointer sized unsigned integer",
                        )
                    })?;

                    let mut string_bytes = Vec::new();
                    string_bytes.try_reserve(length).map_err(|_| {
                        io_error!(OutOfMemory, "allocation of memory for string value failed")
                    })?;
                    string_bytes.extend((0..length).map(|_| 0));

                    read.read_exact(&mut string_bytes).await?;

                    ValueLeaf::String(String::from_utf8(string_bytes).map_err(|_| {
                        io_error!(InvalidData, "allocation of memory for string value failed")
                    })?)
                }
                SchemaLeaf::Uint32 => ValueLeaf::Uint32(read.read_u32().await?),
                SchemaLeaf::Boolean => ValueLeaf::Boolean(read.read_u8().await? != 0),
            }),
        })
    }

    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        match self {
            Value::Product(fields) => {
                for field in fields {
                    Box::pin(field.write(write)).await?;
                }
            }
            Value::Sum(discriminant, variant) => {
                write.write_u32(*discriminant).await?;
                Box::pin(variant.write(write)).await?;
            }
            Value::List(values) => {
                write
                    .write_u32(values.len().try_into().map_err(|_| {
                        io_error!(
                            OutOfMemory,
                            "list value length doesn't fit into a 32 bit unsigned integer",
                        )
                    })?)
                    .await?;

                for value in values {
                    Box::pin(value.write(write)).await?;
                }
            }
            Value::Leaf(leaf) => match leaf {
                ValueLeaf::String(value) => {
                    write
                        .write_u32(value.len().try_into().map_err(|_| {
                            io_error!(
                                OutOfMemory,
                                "string value length doesn't fit into a 32 bit unsigned integer",
                            )
                        })?)
                        .await?;

                    write.write_all(value.as_bytes()).await?;
                }
                ValueLeaf::Uint32(value) => write.write_u32(*value).await?,
                ValueLeaf::Boolean(value) => write.write_u8(*value as u8).await?,
            },
        }

        Ok(())
    }
}
