use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::io_error;

#[derive(Debug)]
pub enum SchemaNode {
    Product(Vec<SchemaNode>),
    Sum(Vec<SchemaNode>),
    List(Box<SchemaNode>),
    Leaf(SchemaLeaf),
}

#[derive(Debug, Clone, Copy)]
pub enum SchemaLeaf {
    String,
    Uint32,
    Boolean,
    Unit,
}

impl SchemaNode {
    fn discriminant(&self) -> u8 {
        match self {
            Self::Product(_) => 0,
            Self::Sum(_) => 1,
            Self::List(_) => 2,
            Self::Leaf(SchemaLeaf::String) => 3,
            Self::Leaf(SchemaLeaf::Uint32) => 4,
            Self::Leaf(SchemaLeaf::Boolean) => 5,
            Self::Leaf(SchemaLeaf::Unit) => 6,
        }
    }

    pub async fn read(read: &mut (impl AsyncReadExt + Unpin)) -> io::Result<Self> {
        let discriminant = read.read_u8().await?;

        let node = match discriminant {
            0 => {
                let length: usize = read.read_u32().await?.try_into().map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "product schema field count doesn't fit into a pointer sized unsigned integer",
                    )
                })?;

                let mut fields = Vec::new();
                fields.try_reserve(length).map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "allocation of memory for product schema fields failed"
                    )
                })?;

                for _ in 0..length {
                    fields.push(Box::pin(SchemaNode::read(read)).await.unwrap());
                }

                SchemaNode::Product(fields)
            }
            1 => {
                let length: usize = read.read_u32().await?.try_into().map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "sum schema variant count doesn't fit into a pointer sized unsigned integer",
                    )
                })?;

                let mut variants = Vec::new();
                variants.try_reserve(length).map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "allocation of memory for sum schema variants failed"
                    )
                })?;

                for _ in 0..length {
                    variants.push(Box::pin(SchemaNode::read(read)).await.unwrap());
                }

                SchemaNode::Sum(variants)
            }
            2 => Self::List(Box::new(Box::pin(Self::read(read)).await?)),
            3 => Self::Leaf(SchemaLeaf::String),
            4 => Self::Leaf(SchemaLeaf::Uint32),
            5 => Self::Leaf(SchemaLeaf::Boolean),
            6 => Self::Leaf(SchemaLeaf::Unit),
            _ => {
                return Err(io_error!(
                    InvalidData,
                    "invalid discriminant while parsing schema node",
                ));
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
                        io_error!(
                            OutOfMemory,
                            "product schema field count doesn't fit into a 32 bit unsigned integer",
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
                        io_error!(
                            OutOfMemory,
                            "sum schema variant count doesn't fit into a 32 bit unsigned integer",
                        )
                    })?)
                    .await?;

                for variant in variants {
                    Box::pin(variant.write(write)).await?;
                }
            }
            SchemaNode::List(schema_node) => Box::pin(schema_node.write(write)).await?,
            SchemaNode::Leaf(_) => {}
        }

        Ok(())
    }
}
