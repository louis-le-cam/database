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
    Uint32(u32),
    Boolean(bool),
    Unit,
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
            Value::Product(fields) => fields
                .get(usize::try_from(*segment).ok()?)
                .and_then(|value| Value::scope(value.clone(), segments)),
            Value::Sum(discriminant, value) => (*discriminant == *segment)
                .then(|| Value::scope(value.clone(), segments))
                .flatten(),
            Value::List(list) => list
                .get(usize::try_from(*segment).ok()?)
                .and_then(|value| Value::scope(value.clone(), segments)),
            Value::String(_) | Value::Uint32(_) | Value::Boolean(_) | Value::Unit => None,
        }
    }

    pub fn equal(&self, rhs: &Value) -> bool {
        match (self, rhs) {
            (Value::Product(lhs), Value::Product(rhs)) => {
                debug_assert_eq!(lhs.len(), rhs.len());

                lhs.iter()
                    .zip(rhs)
                    .all(|(lhs, rhs)| lhs.lock().unwrap().equal(&rhs.lock().unwrap()))
            }
            (Value::Sum(lhs_discriminant, lhs), Value::Sum(rhs_discriminant, rhs)) => {
                (lhs_discriminant == rhs_discriminant)
                    && lhs.lock().unwrap().equal(&rhs.lock().unwrap())
            }
            (Value::List(lhs), Value::List(rhs)) => {
                lhs.len() == rhs.len()
                    && lhs
                        .iter()
                        .zip(rhs)
                        .all(|(lhs, rhs)| lhs.lock().unwrap().equal(&rhs.lock().unwrap()))
            }
            (Value::String(lhs), Value::String(rhs)) => lhs == rhs,
            (Value::Uint32(lhs), Value::Uint32(rhs)) => lhs == rhs,
            (Value::Boolean(lhs), Value::Boolean(rhs)) => lhs == rhs,
            (Value::Unit, Value::Unit) => true,
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
                        Box::pin(Value::read(field, read)).await?,
                    )));
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
                    Arc::new(Mutex::new(Box::pin(Value::read(variant, read)).await?)),
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
                        Box::pin(Value::read(inner, read)).await?,
                    )));
                }

                Value::List(values)
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

                Value::String(String::from_utf8(string_bytes).map_err(|_| {
                    io_error!(InvalidData, "allocation of memory for string value failed")
                })?)
            }
            SchemaNode::Uint32 => Value::Uint32(read.read_u32().await?),
            SchemaNode::Boolean => Value::Boolean(read.read_u8().await? != 0),
            SchemaNode::Unit => Value::Unit,
        })
    }

    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        match self {
            Value::Product(fields) => {
                for field in fields {
                    Box::pin(field.lock().unwrap().write(write)).await?;
                }
            }
            Value::Sum(discriminant, variant) => {
                write.write_u32(*discriminant).await?;
                Box::pin(variant.lock().unwrap().write(write)).await?;
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
                    Box::pin(value.lock().unwrap().write(write)).await?;
                }
            }
            Value::String(value) => {
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
            Value::Uint32(value) => write.write_u32(*value).await?,
            Value::Boolean(value) => write.write_u8(*value as u8).await?,
            Value::Unit => {}
        }

        Ok(())
    }
}
