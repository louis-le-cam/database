use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::schema::SchemaNode;

#[derive(Clone, Debug)]
pub enum Value {
    List(Vec<Value>),
    Product(Vec<Value>),
    Sum(u32, Box<Value>),
    U64(u64),
    String(String),
    Boolean(bool),
    Unit,
}

impl Value {
    pub fn scope(self, path: &[u32]) -> Option<Self> {
        let mut value = self;

        for segment in path {
            value = value.scope_one(*segment)?;
        }

        Some(value)
    }

    fn scope_one(self, segment: u32) -> Option<Self> {
        match self {
            Value::List(values) => values.into_iter().nth(segment as usize),
            Value::Product(product) => product.into_iter().nth(segment as usize),
            Value::Sum(discriminant, value) => (discriminant == segment).then_some(*value),
            Value::U64(_) => None,
            Value::String(_) => None,
            Value::Boolean(_) => None,
            Value::Unit => None,
        }
    }

    pub fn scope_ref(&self, path: &[u32]) -> Option<&Self> {
        let mut value = self;

        for segment in path {
            value = value.scope_one_ref(*segment)?;
        }

        Some(value)
    }

    fn scope_one_ref(&self, segment: u32) -> Option<&Self> {
        match self {
            Value::List(values) => values.get(segment as usize),
            Value::Product(product) => product.get(segment as usize),
            Value::Sum(discriminant, value) => (*discriminant == segment).then_some(value.as_ref()),
            Value::U64(_) => None,
            Value::String(_) => None,
            Value::Boolean(_) => None,
            Value::Unit => None,
        }
    }

    pub fn scope_mut(&mut self, path: &[u32]) -> Option<&mut Self> {
        let mut value = self;

        for segment in path {
            value = value.scope_one_mut(*segment)?;
        }

        Some(value)
    }

    fn scope_one_mut(&mut self, segment: u32) -> Option<&mut Self> {
        match self {
            Value::List(values) => values.get_mut(segment as usize),
            Value::Product(product) => product.get_mut(segment as usize),
            Value::Sum(discriminant, value) => (*discriminant == segment).then_some(value.as_mut()),
            Value::U64(_) => None,
            Value::String(_) => None,
            Value::Boolean(_) => None,
            Value::Unit => None,
        }
    }

    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> std::io::Result<()> {
        match self {
            Value::List(values) => {
                write.write_u64(values.len() as u64).await?;

                for value in values {
                    Box::pin(value.write(write)).await?;
                }
            }
            Value::Product(values) => {
                for value in values {
                    Box::pin(value.write(write)).await?;
                }
            }
            Value::Sum(discriminant, value) => {
                write.write_u32(*discriminant).await?;
                Box::pin(value.write(write)).await?;
            }
            Value::U64(value) => write.write_u64(*value).await?,
            Value::String(value) => {
                write.write_u64(value.len() as u64).await?;
                write.write_all(value.as_bytes()).await?;
            }
            Value::Boolean(value) => write.write_u8(*value as u8).await?,
            Value::Unit => {}
        }

        Ok(())
    }

    pub async fn read(
        schema: &SchemaNode,
        read: &mut (impl AsyncReadExt + Unpin),
    ) -> std::io::Result<Self> {
        Ok(match schema {
            SchemaNode::List(inner_schema) => {
                let length = read.read_u64().await?;
                let mut values = Vec::with_capacity(length as usize);

                for _ in 0..length {
                    values.push(Box::pin(Self::read(inner_schema, read)).await?);
                }

                Value::List(values)
            }
            SchemaNode::Product(product_schema) => {
                let mut values = Vec::with_capacity(product_schema.len());

                for (_, item_schema) in product_schema.iter() {
                    values.push(Box::pin(Self::read(item_schema, read)).await?);
                }

                Value::Product(values)
            }
            SchemaNode::Sum(sum_schema) => {
                let discriminant = read.read_u32().await?;
                let (_, item_schema) = sum_schema.get(discriminant as usize).unwrap();

                Value::Sum(
                    discriminant,
                    Box::new(Box::pin(Self::read(item_schema, read)).await?),
                )
            }
            SchemaNode::U64 => Value::U64(read.read_u64().await?),
            SchemaNode::String => {
                let length = read.read_u64().await?;
                let mut bytes = vec![0; length as usize];
                read.read_exact(bytes.as_mut_slice()).await?;

                Value::String(String::from_utf8(bytes).unwrap())
            }
            SchemaNode::Boolean => Value::Boolean(read.read_u32().await? != 0),
            SchemaNode::Unit => Value::Unit,
        })
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::List(lhs), Value::List(rhs)) => {
                lhs.len() == rhs.len() && lhs.iter().zip(rhs).all(|(lhs, rhs)| lhs == rhs)
            }
            (Value::Product(lhs), Value::Product(rhs)) => {
                assert_eq!(lhs.len(), rhs.len());
                lhs.iter().zip(rhs).all(|(lhs, rhs)| lhs == rhs)
            }
            (Value::Sum(lhs_discriminant, lhs), Value::Sum(rhs_discriminant, rhs)) => {
                lhs_discriminant == rhs_discriminant && lhs == rhs
            }
            (Value::U64(lhs), Value::U64(rhs)) => lhs == rhs,
            (Value::String(lhs), Value::String(rhs)) => lhs == rhs,
            (Value::Boolean(lhs), Value::Boolean(rhs)) => lhs == rhs,
            (Value::Unit, Value::Unit) => true,
            _ => panic!(),
        }
    }
}
