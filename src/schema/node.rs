use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::io_error;

#[derive(Clone, Debug)]
pub enum SchemaNode {
    Product(Vec<SchemaNode>),
    Sum(Vec<SchemaNode>),
    List(Box<SchemaNode>),
    String,
    Boolean,
    Unit,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Float32,
    Float64,
}

pub mod schema_discriminant {
    pub const PRODUCT: u8 = 0;
    pub const SUM: u8 = 1;
    pub const LIST: u8 = 2;
    pub const STRING: u8 = 3;
    pub const BOOLEAN: u8 = 4;
    pub const UNIT: u8 = 5;
    pub const UINT8: u8 = 6;
    pub const UINT16: u8 = 7;
    pub const UINT32: u8 = 8;
    pub const UINT64: u8 = 9;
    pub const UINT128: u8 = 10;
    pub const INT8: u8 = 11;
    pub const INT16: u8 = 12;
    pub const INT32: u8 = 13;
    pub const INT64: u8 = 14;
    pub const INT128: u8 = 15;
    pub const FLOAT32: u8 = 16;
    pub const FLOAT64: u8 = 17;
}

impl SchemaNode {
    fn discriminant(&self) -> u8 {
        match self {
            Self::Product(_) => schema_discriminant::PRODUCT,
            Self::Sum(_) => schema_discriminant::SUM,
            Self::List(_) => schema_discriminant::LIST,
            Self::String => schema_discriminant::STRING,
            Self::Boolean => schema_discriminant::BOOLEAN,
            Self::Unit => schema_discriminant::UNIT,
            Self::Uint8 => schema_discriminant::UINT8,
            Self::Uint16 => schema_discriminant::UINT16,
            Self::Uint32 => schema_discriminant::UINT32,
            Self::Uint64 => schema_discriminant::UINT64,
            Self::Uint128 => schema_discriminant::UINT128,
            Self::Int8 => schema_discriminant::INT8,
            Self::Int16 => schema_discriminant::INT16,
            Self::Int32 => schema_discriminant::INT32,
            Self::Int64 => schema_discriminant::INT64,
            Self::Int128 => schema_discriminant::INT128,
            Self::Float32 => schema_discriminant::FLOAT32,
            Self::Float64 => schema_discriminant::FLOAT64,
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
            schema_discriminant::BOOLEAN => Self::Boolean,
            schema_discriminant::UNIT => Self::Unit,
            schema_discriminant::UINT8 => Self::Uint8,
            schema_discriminant::UINT16 => Self::Uint16,
            schema_discriminant::UINT32 => Self::Uint32,
            schema_discriminant::UINT64 => Self::Uint64,
            schema_discriminant::UINT128 => Self::Uint128,
            schema_discriminant::INT8 => Self::Int8,
            schema_discriminant::INT16 => Self::Int16,
            schema_discriminant::INT32 => Self::Int32,
            schema_discriminant::INT64 => Self::Int64,
            schema_discriminant::INT128 => Self::Int128,
            schema_discriminant::FLOAT32 => Self::Float32,
            schema_discriminant::FLOAT64 => Self::Float64,
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
            Self::Product(fields) => {
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
            Self::Sum(variants) => {
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
            Self::List(schema_node) => Box::pin(schema_node.write(write)).await?,
            Self::String
            | Self::Boolean
            | Self::Unit
            | Self::Uint8
            | Self::Uint16
            | Self::Uint32
            | Self::Uint64
            | Self::Uint128
            | Self::Int8
            | Self::Int16
            | Self::Int32
            | Self::Int64
            | Self::Int128
            | Self::Float32
            | Self::Float64 => {}
        }

        Ok(())
    }
}
