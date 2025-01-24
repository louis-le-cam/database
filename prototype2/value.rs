use std::{borrow::Cow, io};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{BoxOrRef, SchemaNode};

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Product(Cow<'a, [Value<'a>]>),
    Sum(u32, BoxOrRef<'a, Value<'a>>),
    List(Cow<'a, [Value<'a>]>),
    String(Cow<'a, str>),
    Boolean(bool),
    Float64(f64),
    Float32(f32),
    Uint128(u128),
    Uint64(u64),
    Uint32(u32),
    Uint16(u16),
    Uint8(u8),
    Int128(i128),
    Int64(i64),
    Int32(i32),
    Int16(i16),
    Int8(i8),
    Unit,
}

impl Value<'_> {
    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        match self {
            Value::Product(values) => {
                for value in values.iter() {
                    Box::pin(value.write(write)).await?;
                }
            }
            Value::Sum(discriminant, value) => {
                write.write_u32(*discriminant).await?;
                Box::pin(value.write(write)).await?;
            }
            Value::List(values) => {
                write.write_u64(values.len().try_into().unwrap()).await?;

                for value in values.iter() {
                    Box::pin(value.write(write)).await?;
                }
            }
            Value::String(value) => {
                write.write_u64(value.len().try_into().unwrap()).await?;
                write.write_all(value.as_bytes()).await?;
            }
            Value::Boolean(value) => write.write_u8(*value as u8).await?,
            Value::Float64(value) => write.write_f64(*value).await?,
            Value::Float32(value) => write.write_f32(*value).await?,
            Value::Uint128(value) => write.write_u128(*value).await?,
            Value::Uint64(value) => write.write_u64(*value).await?,
            Value::Uint32(value) => write.write_u32(*value).await?,
            Value::Uint16(value) => write.write_u16(*value).await?,
            Value::Uint8(value) => write.write_u8(*value).await?,
            Value::Int128(value) => write.write_i128(*value).await?,
            Value::Int64(value) => write.write_i64(*value).await?,
            Value::Int32(value) => write.write_i32(*value).await?,
            Value::Int16(value) => write.write_i16(*value).await?,
            Value::Int8(value) => write.write_i8(*value).await?,
            Value::Unit => {}
        }

        Ok(())
    }

    pub async fn read(
        schema: &SchemaNode<'_>,
        read: &mut (impl AsyncReadExt + Unpin),
    ) -> io::Result<Self> {
        Ok(match schema {
            SchemaNode::Product(fields_schemas) => {
                let mut fields = Vec::with_capacity(fields_schemas.len().try_into().unwrap());

                for (_, field_schema) in fields_schemas.iter() {
                    fields.push(Box::pin(Value::read(field_schema, read)).await?);
                }

                Value::Product(Cow::Owned(fields))
            }
            SchemaNode::Sum(variants_schemas) => {
                let discriminant = read.read_u32().await?;

                let Some((_, variant_schema)) =
                    variants_schemas.get::<usize>(discriminant.try_into().unwrap())
                else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid discriminant value for sum type",
                    ));
                };

                Value::Sum(
                    discriminant,
                    BoxOrRef::Box(Box::new(Box::pin(Value::read(variant_schema, read)).await?)),
                )
            }
            SchemaNode::List(inner_schema) => {
                let length = read.read_u64().await?;
                let mut values = Vec::with_capacity(length.try_into().unwrap());

                for _ in 0..length {
                    values.push(Box::pin(Value::read(inner_schema, read)).await?);
                }

                Value::List(Cow::Owned(values))
            }
            SchemaNode::String => {
                let length = read.read_u64().await?;
                let mut bytes = vec![0; length.try_into().unwrap()];
                read.read_exact(&mut bytes).await?;

                Value::String(Cow::Owned(String::from_utf8(bytes).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "non utf8 string")
                })?))
            }
            SchemaNode::Boolean => Value::Boolean(read.read_u8().await? != 0),
            SchemaNode::Float64 => Value::Float64(read.read_f64().await?),
            SchemaNode::Float32 => Value::Float32(read.read_f32().await?),
            SchemaNode::Uint128 => Value::Uint128(read.read_u128().await?),
            SchemaNode::Uint64 => Value::Uint64(read.read_u64().await?),
            SchemaNode::Uint32 => Value::Uint32(read.read_u32().await?),
            SchemaNode::Uint16 => Value::Uint16(read.read_u16().await?),
            SchemaNode::Uint8 => Value::Uint8(read.read_u8().await?),
            SchemaNode::Int128 => Value::Int128(read.read_i128().await?),
            SchemaNode::Int64 => Value::Int64(read.read_i64().await?),
            SchemaNode::Int32 => Value::Int32(read.read_i32().await?),
            SchemaNode::Int16 => Value::Int16(read.read_i16().await?),
            SchemaNode::Int8 => Value::Int8(read.read_i8().await?),
            SchemaNode::Unit => Value::Unit,
        })
    }

    // TODO: Not very efficient, should not be used
    pub fn into_owned(self) -> Value<'static> {
        match self {
            Value::Product(values) => Value::Product(Cow::Owned(
                values
                    .into_owned()
                    .into_iter()
                    .map(|value| value.into_owned())
                    .collect(),
            )),
            Value::Sum(discriminant, value) => Value::Sum(
                discriminant,
                BoxOrRef::Box(Box::new((*value).clone().into_owned())),
            ),
            Value::List(values) => Value::List(Cow::Owned(
                values
                    .into_owned()
                    .into_iter()
                    .map(|value| value.into_owned())
                    .collect(),
            )),
            Value::String(value) => Value::String(Cow::Owned(value.into_owned())),
            Value::Boolean(value) => Value::Boolean(value),
            Value::Float64(value) => Value::Float64(value),
            Value::Float32(value) => Value::Float32(value),
            Value::Uint128(value) => Value::Uint128(value),
            Value::Uint64(value) => Value::Uint64(value),
            Value::Uint32(value) => Value::Uint32(value),
            Value::Uint16(value) => Value::Uint16(value),
            Value::Uint8(value) => Value::Uint8(value),
            Value::Int128(value) => Value::Int128(value),
            Value::Int64(value) => Value::Int64(value),
            Value::Int32(value) => Value::Int32(value),
            Value::Int16(value) => Value::Int16(value),
            Value::Int8(value) => Value::Int8(value),
            Value::Unit => Value::Unit,
        }
    }
}
