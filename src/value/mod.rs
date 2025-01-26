use std::{
    io,
    sync::{Arc, Mutex},
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{io_error, SchemaNode};

#[derive(Debug, Clone)]
pub enum Value {
    Product(Vec<Arc<Mutex<Value>>>),
    Sum(u32, Arc<Mutex<Value>>),
    List(Vec<Arc<Mutex<Value>>>),
    String(String),
    Boolean(bool),
    Unit,
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Uint128(u128),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),
    Float32(f32),
    Float64(f64),
}

impl Value {
    pub fn scope_scopes(scopes: Vec<Arc<Mutex<Self>>>, path: &[u32]) -> Option<Arc<Mutex<Self>>> {
        let Some((segment, segments)) = path.split_first() else {
            return None;
        };

        scopes
            .get(*segment as usize)
            .and_then(|value| Value::scope(value.clone(), segments))
    }

    pub fn scope(value: Arc<Mutex<Self>>, path: &[u32]) -> Option<Arc<Mutex<Self>>> {
        let Some((segment, segments)) = path.split_first() else {
            return Some(value.clone());
        };

        match &*value.lock().unwrap() {
            Self::Product(fields) => fields
                .get(usize::try_from(*segment).ok()?)
                .and_then(|value| Self::scope(value.clone(), segments)),
            Self::Sum(discriminant, value) => (*discriminant == *segment)
                .then(|| Self::scope(value.clone(), segments))
                .flatten(),
            Self::List(list) => list
                .get(usize::try_from(*segment).ok()?)
                .and_then(|value| Self::scope(value.clone(), segments)),
            Self::String(_)
            | Self::Boolean(_)
            | Self::Unit
            | Self::Uint8(_)
            | Self::Uint16(_)
            | Self::Uint32(_)
            | Self::Uint64(_)
            | Self::Uint128(_)
            | Self::Int8(_)
            | Self::Int16(_)
            | Self::Int32(_)
            | Self::Int64(_)
            | Self::Int128(_)
            | Self::Float32(_)
            | Self::Float64(_) => None,
        }
    }

    pub fn equal(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Product(lhs), Self::Product(rhs)) => {
                debug_assert_eq!(lhs.len(), rhs.len());

                lhs.iter()
                    .zip(rhs)
                    .all(|(lhs, rhs)| lhs.lock().unwrap().equal(&rhs.lock().unwrap()))
            }
            (Self::Sum(lhs_discriminant, lhs), Self::Sum(rhs_discriminant, rhs)) => {
                (lhs_discriminant == rhs_discriminant)
                    && lhs.lock().unwrap().equal(&rhs.lock().unwrap())
            }
            (Self::List(lhs), Self::List(rhs)) => {
                lhs.len() == rhs.len()
                    && lhs
                        .iter()
                        .zip(rhs)
                        .all(|(lhs, rhs)| lhs.lock().unwrap().equal(&rhs.lock().unwrap()))
            }
            (Self::String(lhs), Self::String(rhs)) => lhs == rhs,
            (Self::Uint8(lhs), Self::Uint8(rhs)) => lhs == rhs,
            (Self::Uint16(lhs), Self::Uint16(rhs)) => lhs == rhs,
            (Self::Uint32(lhs), Self::Uint32(rhs)) => lhs == rhs,
            (Self::Uint64(lhs), Self::Uint64(rhs)) => lhs == rhs,
            (Self::Uint128(lhs), Self::Uint128(rhs)) => lhs == rhs,
            (Self::Int8(lhs), Self::Int8(rhs)) => lhs == rhs,
            (Self::Int16(lhs), Self::Int16(rhs)) => lhs == rhs,
            (Self::Int32(lhs), Self::Int32(rhs)) => lhs == rhs,
            (Self::Int64(lhs), Self::Int64(rhs)) => lhs == rhs,
            (Self::Int128(lhs), Self::Int128(rhs)) => lhs == rhs,
            (Self::Float32(lhs), Self::Float32(rhs)) => lhs == rhs,
            (Self::Float64(lhs), Self::Float64(rhs)) => lhs == rhs,
            (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs == rhs,
            (Self::Unit, Self::Unit) => true,
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
                    values.push(Arc::new(Mutex::new(
                        Box::pin(Self::read(field, read)).await?,
                    )));
                }

                Self::Product(values)
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

                Self::Sum(
                    discriminant,
                    Arc::new(Mutex::new(Box::pin(Self::read(variant, read)).await?)),
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
                    values.push(Arc::new(Mutex::new(
                        Box::pin(Self::read(inner, read)).await?,
                    )));
                }

                Self::List(values)
            }
            SchemaNode::String => {
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

                Self::String(String::from_utf8(string_bytes).map_err(|_| {
                    io_error!(InvalidData, "allocation of memory for string value failed")
                })?)
            }
            SchemaNode::Boolean => Self::Boolean(read.read_u8().await? != 0),
            SchemaNode::Unit => Self::Unit,
            SchemaNode::Uint8 => Self::Uint8(read.read_u8().await?),
            SchemaNode::Uint16 => Self::Uint16(read.read_u16().await?),
            SchemaNode::Uint32 => Self::Uint32(read.read_u32().await?),
            SchemaNode::Uint64 => Self::Uint64(read.read_u64().await?),
            SchemaNode::Uint128 => Self::Uint128(read.read_u128().await?),
            SchemaNode::Int8 => Self::Int8(read.read_i8().await?),
            SchemaNode::Int16 => Self::Int16(read.read_i16().await?),
            SchemaNode::Int32 => Self::Int32(read.read_i32().await?),
            SchemaNode::Int64 => Self::Int64(read.read_i64().await?),
            SchemaNode::Int128 => Self::Int128(read.read_i128().await?),
            SchemaNode::Float32 => Self::Float32(read.read_f32().await?),
            SchemaNode::Float64 => Self::Float64(read.read_f64().await?),
        })
    }

    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        match self {
            Self::Product(fields) => {
                for field in fields {
                    Box::pin(field.lock().unwrap().write(write)).await?;
                }
            }
            Self::Sum(discriminant, variant) => {
                write.write_u32(*discriminant).await?;
                Box::pin(variant.lock().unwrap().write(write)).await?;
            }
            Self::List(values) => {
                write
                    .write_u32(values.len().try_into().map_err(|_| {
                        io_error!(
                            OutOfMemory,
                            "list value length doesn't fit into a 32 bit unsigned integer",
                        )
                    })?)
                    .await?;

                for value in values {
                    Box::pin(value.lock().unwrap().write(write)).await?;
                }
            }
            Self::String(value) => {
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
            Self::Boolean(value) => write.write_u8(*value as u8).await?,
            Self::Unit => {}
            Self::Uint8(value) => write.write_u8(*value).await?,
            Self::Uint16(value) => write.write_u16(*value).await?,
            Self::Uint32(value) => write.write_u32(*value).await?,
            Self::Uint64(value) => write.write_u64(*value).await?,
            Self::Uint128(value) => write.write_u128(*value).await?,
            Self::Int8(value) => write.write_i8(*value).await?,
            Self::Int16(value) => write.write_i16(*value).await?,
            Self::Int32(value) => write.write_i32(*value).await?,
            Self::Int64(value) => write.write_i64(*value).await?,
            Self::Int128(value) => write.write_i128(*value).await?,
            Self::Float32(value) => write.write_f32(*value).await?,
            Self::Float64(value) => write.write_f64(*value).await?,
        }

        Ok(())
    }
}
