use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::io_error;

#[derive(Debug)]
pub enum SchemaNode {
    Product(Vec<SchemaNode>),
    Sum(Vec<SchemaNode>),
    List(Box<SchemaNode>),
    String,
    Uint32,
    Boolean,
    Unit,
}

pub mod schema_discriminant {
    pub const PRODUCT: u8 = 0;
    pub const SUM: u8 = 1;
    pub const LIST: u8 = 2;
    pub const STRING: u8 = 3;
    pub const UINT32: u8 = 4;
    pub const BOOLEAN: u8 = 5;
    pub const UNIT: u8 = 6;
}

impl SchemaNode {
    fn discriminant(&self) -> u8 {
        match self {
            Self::Product(_) => schema_discriminant::PRODUCT,
            Self::Sum(_) => schema_discriminant::SUM,
            Self::List(_) => schema_discriminant::LIST,
            Self::String => schema_discriminant::STRING,
            Self::Uint32 => schema_discriminant::UINT32,
            Self::Boolean => schema_discriminant::BOOLEAN,
            Self::Unit => schema_discriminant::UNIT,
        }
    }

    pub async fn read(read: &mut (impl AsyncReadExt + Unpin)) -> io::Result<Self> {
        let discriminant = read.read_u8().await?;

        let node = match discriminant {
            schema_discriminant::PRODUCT => {
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
            schema_discriminant::SUM => {
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
            schema_discriminant::LIST => Self::List(Box::new(Box::pin(Self::read(read)).await?)),
            schema_discriminant::STRING => Self::String,
            schema_discriminant::UINT32 => Self::Uint32,
            schema_discriminant::BOOLEAN => Self::Boolean,
            schema_discriminant::UNIT => Self::Unit,
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
            SchemaNode::String | SchemaNode::Uint32 | SchemaNode::Boolean | SchemaNode::Unit => {}
        }

        Ok(())
    }
}
