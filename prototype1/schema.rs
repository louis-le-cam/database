use std::marker::Unpin;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(FromPrimitive)]
pub enum SchemaNodeKind {
    List,
    Product,
    Sum,
    U64,
    String,
    Boolean,
    Unit,
}

#[derive(Clone, Debug)]
pub enum SchemaNode {
    List(Box<SchemaNode>),
    Product(Vec<(String, SchemaNode)>),
    Sum(Vec<(String, SchemaNode)>),
    U64,
    String,
    Boolean,
    Unit,
}

impl SchemaNodeKind {
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    pub fn from_u16(number: u16) -> Option<Self> {
        FromPrimitive::from_u16(number)
    }
}

impl SchemaNode {
    pub fn scope_ref(&self, path: &[u32]) -> Option<&Self> {
        let mut schema = self;

        for segment in path {
            schema = schema.scope_one_ref(*segment)?;
        }

        Some(schema)
    }

    fn scope_one_ref(&self, segment: u32) -> Option<&Self> {
        match self {
            SchemaNode::List(schema) => Some(schema.as_ref()),
            SchemaNode::Product(product) => product.get(segment as usize).map(|(_, schema)| schema),
            SchemaNode::Sum(sum) => sum.get(segment as usize).map(|(_, schema)| schema),
            SchemaNode::U64 => None,
            SchemaNode::String => None,
            SchemaNode::Boolean => None,
            SchemaNode::Unit => None,
        }
    }

    pub fn scope_mut(&mut self, path: &[u32]) -> Option<&mut Self> {
        let mut schema = self;

        for segment in path {
            schema = schema.scope_one_mut(*segment)?;
        }

        Some(schema)
    }

    fn scope_one_mut(&mut self, segment: u32) -> Option<&mut Self> {
        match self {
            SchemaNode::List(schema) => Some(schema.as_mut()),
            SchemaNode::Product(product) => {
                product.get_mut(segment as usize).map(|(_, schema)| schema)
            }
            SchemaNode::Sum(sum) => sum.get_mut(segment as usize).map(|(_, schema)| schema),
            SchemaNode::U64 => None,
            SchemaNode::String => None,
            SchemaNode::Boolean => None,
            SchemaNode::Unit => None,
        }
    }

    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> std::io::Result<()> {
        let mut nodes = Vec::<(Option<&String>, &SchemaNode)>::from([(None, self)]);

        while let Some((name, node)) = nodes.last() {
            if let Some(name) = name {
                write.write_u32(name.len().try_into().unwrap()).await?;
                write.write_all(name.as_bytes()).await?;
            }

            let kind = match node {
                SchemaNode::List(_) => 0,
                SchemaNode::Product(_) => 1,
                SchemaNode::Sum(_) => 2,
                SchemaNode::U64 => 3,
                SchemaNode::String => 4,
                SchemaNode::Boolean => 5,
                SchemaNode::Unit => 6,
            };

            write.write_u8(kind).await?;

            match node {
                SchemaNode::List(inner) => *nodes.last_mut().unwrap() = (None, inner),
                SchemaNode::Product(product) => {
                    write.write_u32(product.len().try_into().unwrap()).await?;
                    nodes
                        .splice(
                            nodes.len() - 1..nodes.len(),
                            product.iter().rev().map(|(name, node)| (Some(name), node)),
                        )
                        .count();
                }
                SchemaNode::Sum(sum) => {
                    write.write_u32(sum.len().try_into().unwrap()).await?;
                    nodes
                        .splice(
                            nodes.len() - 1..nodes.len(),
                            sum.iter().rev().map(|(name, node)| (Some(name), node)),
                        )
                        .count();
                }
                SchemaNode::U64 | SchemaNode::String | SchemaNode::Boolean | SchemaNode::Unit => {
                    nodes.pop();
                }
            };
        }

        Ok(())
    }

    pub async fn read(read: &mut (impl AsyncReadExt + Unpin)) -> std::io::Result<Self> {
        Ok(match read.read_u8().await? {
            0 => SchemaNode::List(Box::new(Box::pin(Self::read(read)).await?)),
            1 => {
                let length = read.read_u32().await?;

                let mut product = Vec::with_capacity(length as usize);

                for _ in 0..length {
                    let name_length = read.read_u32().await?;
                    let mut name_bytes = vec![0; name_length as usize];
                    read.read_exact(name_bytes.as_mut_slice()).await?;

                    product.push((
                        String::from_utf8(name_bytes).unwrap(),
                        Box::pin(Self::read(read)).await?,
                    ));
                }

                SchemaNode::Product(product)
            }
            2 => {
                let length = read.read_u32().await?;

                let mut sum = Vec::with_capacity(length as usize);

                for _ in 0..length {
                    let name_length = read.read_u32().await?;
                    let mut name_bytes = vec![0; name_length as usize];
                    read.read_exact(name_bytes.as_mut_slice()).await?;

                    sum.push((
                        String::from_utf8(name_bytes).unwrap(),
                        Box::pin(Self::read(read)).await?,
                    ));
                }

                SchemaNode::Sum(sum)
            }
            3 => SchemaNode::U64,
            4 => SchemaNode::String,
            5 => SchemaNode::Boolean,
            6 => SchemaNode::Unit,
            _ => panic!(),
        })
    }
}
