use std::{
    io,
    sync::{Arc, Mutex},
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{io_error, SchemaNode, Value};

#[derive(Debug, Clone)]
pub enum ExpressionNode {
    Path(Vec<u32>),
    Value(SchemaNode, Value),
    Set(Box<(ExpressionNode, ExpressionNode)>),
    Equal(Box<(ExpressionNode, ExpressionNode)>),
    Filter(Box<(ExpressionNode, ExpressionNode)>),
    Map(Box<(ExpressionNode, ExpressionNode)>),
    And(Box<(ExpressionNode, ExpressionNode)>),
    MapVariant(Box<(ExpressionNode, u32, ExpressionNode)>),
    Fuse(Box<ExpressionNode>),
    Chain(Box<(ExpressionNode, ExpressionNode)>),
    Get(Box<(ExpressionNode, ExpressionNode)>),
    Condition(Box<(ExpressionNode, ExpressionNode, ExpressionNode)>),
    Product(Vec<ExpressionNode>),
    Sum(Box<(u32, ExpressionNode)>),
    List(Vec<ExpressionNode>),
}

pub mod expression_discriminant {
    pub const PATH: u8 = 0;
    pub const VALUE: u8 = 1;
    pub const SET: u8 = 2;
    pub const EQUAL: u8 = 3;
    pub const FILTER: u8 = 4;
    pub const MAP: u8 = 5;
    pub const AND: u8 = 6;
    pub const MAP_VARIANT: u8 = 7;
    pub const FUSE: u8 = 8;
    pub const CHAIN: u8 = 9;
    pub const GET: u8 = 10;
    pub const CONDITION: u8 = 11;
    pub const PRODUCT: u8 = 12;
    pub const SUM: u8 = 13;
    pub const LIST: u8 = 14;
}

impl ExpressionNode {
    pub fn evaluate(self, scopes: Vec<Arc<Mutex<Value>>>) -> Arc<Mutex<Value>> {
        match self {
            ExpressionNode::Path(path) => Value::scope_scopes(scopes, &path).unwrap(),
            ExpressionNode::Value(_, value) => Arc::new(Mutex::new(value)),
            ExpressionNode::Set(operands) => {
                let (left_expression, right_expression) = *operands;

                *left_expression.evaluate(scopes).lock().unwrap() = right_expression
                    .evaluate(scopes.clone())
                    .lock()
                    .unwrap()
                    .clone();

                Arc::new(Mutex::new(Value::Unit))
            }
            ExpressionNode::Equal(operands) => {
                let (left_expression, right_expression) = *operands;

                let left_value = left_expression.evaluate(scopes.clone());
                let right_value = right_expression.evaluate(scopes);

                Arc::new(Mutex::new(Value::Boolean(
                    left_value
                        .clone()
                        .lock()
                        .unwrap()
                        .equal(&right_value.clone().lock().unwrap()),
                )))
            }
            ExpressionNode::Filter(operands) => {
                let (left_expression, right_expression) = *operands;

                let left_value = left_expression.evaluate(scopes.clone());

                let Value::List(values) = &*left_value.lock().unwrap() else {
                    panic!()
                };

                Arc::new(Mutex::new(Value::List(
                    values
                        .iter()
                        .filter(|value| {
                            match *right_expression
                                .clone()
                                .evaluate(
                                    scopes.iter().cloned().chain([(**value).clone()]).collect(),
                                )
                                .lock()
                                .unwrap()
                            {
                                Value::Boolean(keep) => keep,
                                _ => panic!(),
                            }
                        })
                        .cloned()
                        .collect(),
                )))
            }
            ExpressionNode::Map(operands) => {
                let (left_expression, right_expression) = *operands;

                let left_value = left_expression.evaluate(scopes.clone());

                let Value::List(values) = &*left_value.lock().unwrap() else {
                    panic!()
                };

                Arc::new(Mutex::new(Value::List(
                    values
                        .iter()
                        .map(|value| {
                            right_expression
                                .clone()
                                .evaluate(scopes.iter().cloned().chain([value.clone()]).collect())
                        })
                        .collect(),
                )))
            }
            ExpressionNode::And(operands) => {
                let (left_expression, right_expression) = *operands;

                let Value::Boolean(lhs) = *left_expression.evaluate(scopes.clone()).lock().unwrap()
                else {
                    panic!();
                };
                let Value::Boolean(rhs) =
                    *right_expression.evaluate(scopes.clone()).lock().unwrap()
                else {
                    panic!();
                };

                Arc::new(Mutex::new(Value::Boolean(lhs && rhs)))
            }
            ExpressionNode::MapVariant(operands) => {
                let (left_expression, target_discriminant, right_expression) = *operands;

                let lhs = left_expression.evaluate(scopes.clone());

                let Value::Sum(discriminant, variant) = &*lhs.lock().unwrap() else {
                    panic!();
                };

                if *discriminant == target_discriminant {
                    Arc::new(Mutex::new(Value::Sum(
                        *discriminant,
                        right_expression
                            .evaluate(scopes.iter().cloned().chain([variant.clone()]).collect()),
                    )))
                } else {
                    lhs.clone()
                }
            }
            ExpressionNode::Fuse(operand) => {
                let value = operand.evaluate(scopes);
                let Value::Sum(_, inner) = &*value.lock().unwrap() else {
                    panic!()
                };

                inner.clone()
            }
            ExpressionNode::Chain(operands) => {
                let (left_expression, right_expression) = *operands;

                left_expression.evaluate(scopes.clone());
                right_expression.evaluate(scopes)
            }
            ExpressionNode::Get(operands) => {
                let (left_expression, right_expression) = *operands;

                let index = {
                    let rhs = right_expression.evaluate(scopes.clone());
                    let Value::Uint32(index) = &*rhs.lock().unwrap() else {
                        panic!();
                    };

                    *index
                };

                let lhs = left_expression.evaluate(scopes);
                let lhs = lhs.lock().unwrap();
                match &*lhs {
                    Value::Product(product) => product[index as usize].clone(),
                    Value::List(list) => Arc::new(Mutex::new(match list.get(index as usize) {
                        Some(value) => Value::Sum(1, value.clone()),
                        None => Value::Sum(0, Arc::new(Mutex::new(Value::Unit))),
                    })),
                    _ => panic!(),
                }
            }
            ExpressionNode::Condition(operands) => {
                let (condition, if_branch, else_branch) = *operands;

                match &*condition.evaluate(scopes.clone()).lock().unwrap() {
                    Value::Boolean(true) => if_branch.evaluate(scopes),
                    Value::Boolean(false) => else_branch.evaluate(scopes),
                    _ => panic!(),
                }
            }
            ExpressionNode::Product(fields) => Arc::new(Mutex::new(Value::Product(
                fields
                    .into_iter()
                    .map(|field| field.evaluate(scopes.clone()))
                    .collect(),
            ))),
            ExpressionNode::Sum(operands) => {
                let (discriminant, expression) = *operands;

                Arc::new(Mutex::new(Value::Sum(
                    discriminant,
                    expression.evaluate(scopes),
                )))
            }
            ExpressionNode::List(elements) => Arc::new(Mutex::new(Value::List(
                elements
                    .into_iter()
                    .map(|element| element.evaluate(scopes.clone()))
                    .collect(),
            ))),
        }
    }

    fn discriminant(&self) -> u8 {
        match self {
            ExpressionNode::Path(_) => expression_discriminant::PATH,
            ExpressionNode::Value(_, _) => expression_discriminant::VALUE,
            ExpressionNode::Set(_) => expression_discriminant::SET,
            ExpressionNode::Equal(_) => expression_discriminant::EQUAL,
            ExpressionNode::Filter(_) => expression_discriminant::FILTER,
            ExpressionNode::Map(_) => expression_discriminant::MAP,
            ExpressionNode::And(_) => expression_discriminant::AND,
            ExpressionNode::MapVariant(_) => expression_discriminant::MAP_VARIANT,
            ExpressionNode::Fuse(_) => expression_discriminant::FUSE,
            ExpressionNode::Chain(_) => expression_discriminant::CHAIN,
            ExpressionNode::Get(_) => expression_discriminant::GET,
            ExpressionNode::Condition(_) => expression_discriminant::CONDITION,
            ExpressionNode::Product(_) => expression_discriminant::PRODUCT,
            ExpressionNode::Sum(_) => expression_discriminant::SUM,
            ExpressionNode::List(_) => expression_discriminant::LIST,
        }
    }

    pub async fn read(read: &mut (impl AsyncReadExt + Unpin)) -> io::Result<Self> {
        let discriminant = read.read_u8().await?;

        let node = match discriminant {
            expression_discriminant::PATH => {
                let length: usize = read.read_u32().await?.try_into().map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "path expression length doesn't fit into a pointer sized unsigned integer",
                    )
                })?;

                let mut path = Vec::new();
                path.try_reserve(length).map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "allocation of memory for path expression failed"
                    )
                })?;

                for _ in 0..length {
                    path.push(read.read_u32().await?);
                }

                Self::Path(path)
            }
            expression_discriminant::VALUE => {
                let schema = SchemaNode::read(read).await?;
                let value = Value::read(&schema, read).await?;
                Self::Value(schema, value)
            }
            expression_discriminant::SET => Self::Set(Box::new((
                Box::pin(Self::read(read)).await?,
                Box::pin(Self::read(read)).await?,
            ))),
            expression_discriminant::EQUAL => Self::Equal(Box::new((
                Box::pin(Self::read(read)).await?,
                Box::pin(Self::read(read)).await?,
            ))),
            expression_discriminant::FILTER => Self::Filter(Box::new((
                Box::pin(Self::read(read)).await?,
                Box::pin(Self::read(read)).await?,
            ))),
            expression_discriminant::MAP => Self::Map(Box::new((
                Box::pin(Self::read(read)).await?,
                Box::pin(Self::read(read)).await?,
            ))),
            expression_discriminant::AND => Self::And(Box::new((
                Box::pin(Self::read(read)).await?,
                Box::pin(Self::read(read)).await?,
            ))),
            expression_discriminant::MAP_VARIANT => Self::MapVariant(Box::new((
                Box::pin(Self::read(read)).await?,
                read.read_u32().await?,
                Box::pin(Self::read(read)).await?,
            ))),
            expression_discriminant::FUSE => {
                Self::Fuse(Box::new(Box::pin(Self::read(read)).await?))
            }
            expression_discriminant::CHAIN => Self::Chain(Box::new((
                Box::pin(Self::read(read)).await?,
                Box::pin(Self::read(read)).await?,
            ))),
            expression_discriminant::GET => Self::Get(Box::new((
                Box::pin(Self::read(read)).await?,
                Box::pin(Self::read(read)).await?,
            ))),
            expression_discriminant::CONDITION => Self::Condition(Box::new((
                Box::pin(Self::read(read)).await?,
                Box::pin(Self::read(read)).await?,
                Box::pin(Self::read(read)).await?,
            ))),
            expression_discriminant::PRODUCT => {
                let length: usize = read.read_u32().await?.try_into().map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "product expression length doesn't fit into a pointer sized unsigned integer",
                    )
                })?;

                let mut fields = Vec::new();
                fields.try_reserve(length).map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "allocation of memory for product expression failed"
                    )
                })?;

                for _ in 0..length {
                    fields.push(Box::pin(Self::read(read)).await?);
                }

                Self::Product(fields)
            }
            expression_discriminant::SUM => Self::Sum(Box::new((
                read.read_u32().await?,
                Box::pin(Self::read(read)).await?,
            ))),
            expression_discriminant::LIST => {
                let length: usize = read.read_u32().await?.try_into().map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "list expression length doesn't fit into a pointer sized unsigned integer",
                    )
                })?;

                let mut elements = Vec::new();
                elements.try_reserve(length).map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "allocation of memory for list expression failed"
                    )
                })?;

                for _ in 0..length {
                    elements.push(Box::pin(Self::read(read)).await?);
                }

                Self::List(elements)
            }
            _ => {
                return Err(io_error!(
                    InvalidData,
                    "invalid discriminant while parsing expression node",
                ));
            }
        };

        debug_assert_eq!(node.discriminant(), discriminant);

        Ok(node)
    }

    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        write.write_u8(self.discriminant()).await?;

        match self {
            ExpressionNode::Path(segments) => {
                write
                    .write_u32(segments.len().try_into().map_err(|_| {
                        io_error!(
                            OutOfMemory,
                            "path expression length doesn't fit into a 32 bit unsigned integer",
                        )
                    })?)
                    .await?;

                for segment in segments {
                    write.write_u32(*segment).await?;
                }
            }
            ExpressionNode::Value(schema, value) => {
                schema.write(write).await?;
                value.write(write).await?;
            }
            ExpressionNode::Set(operands) => {
                Box::pin(operands.as_ref().0.write(write)).await?;
                Box::pin(operands.as_ref().1.write(write)).await?;
            }
            ExpressionNode::Equal(operands) => {
                Box::pin(operands.as_ref().0.write(write)).await?;
                Box::pin(operands.as_ref().1.write(write)).await?;
            }
            ExpressionNode::Filter(operands) => {
                Box::pin(operands.as_ref().0.write(write)).await?;
                Box::pin(operands.as_ref().1.write(write)).await?;
            }
            ExpressionNode::Map(operands) => {
                Box::pin(operands.as_ref().0.write(write)).await?;
                Box::pin(operands.as_ref().1.write(write)).await?;
            }
            ExpressionNode::And(operands) => {
                Box::pin(operands.as_ref().0.write(write)).await?;
                Box::pin(operands.as_ref().1.write(write)).await?;
            }
            ExpressionNode::MapVariant(operands) => {
                Box::pin(operands.as_ref().0.write(write)).await?;
                write.write_u32(operands.as_ref().1).await?;
                Box::pin(operands.as_ref().2.write(write)).await?;
            }
            ExpressionNode::Fuse(operand) => {
                Box::pin(operand.write(write)).await?;
            }
            ExpressionNode::Chain(operands) => {
                Box::pin(operands.as_ref().0.write(write)).await?;
                Box::pin(operands.as_ref().1.write(write)).await?;
            }
            ExpressionNode::Get(operands) => {
                Box::pin(operands.as_ref().0.write(write)).await?;
                Box::pin(operands.as_ref().1.write(write)).await?;
            }
            ExpressionNode::Condition(operands) => {
                Box::pin(operands.as_ref().0.write(write)).await?;
                Box::pin(operands.as_ref().1.write(write)).await?;
                Box::pin(operands.as_ref().2.write(write)).await?;
            }
            ExpressionNode::Product(fields) => {
                write
                    .write_u32(fields.len().try_into().map_err(|_| {
                        io_error!(
                            OutOfMemory,
                            "product expression length doesn't fit into a 32 bit unsigned integer",
                        )
                    })?)
                    .await?;

                for field in fields {
                    Box::pin(field.write(write)).await?;
                }
            }
            ExpressionNode::Sum(operands) => {
                write.write_u32(operands.as_ref().0).await?;
                Box::pin(operands.as_ref().1.write(write)).await?;
            }
            ExpressionNode::List(elements) => {
                write
                    .write_u32(elements.len().try_into().map_err(|_| {
                        io_error!(
                            OutOfMemory,
                            "product expression length doesn't fit into a 32 bit unsigned integer",
                        )
                    })?)
                    .await?;

                for element in elements {
                    Box::pin(element.write(write)).await?;
                }
            }
        }

        Ok(())
    }
}
