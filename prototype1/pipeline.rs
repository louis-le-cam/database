use std::borrow::Cow;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{read_path, schema::SchemaNode, value::Value, write_path};

#[derive(Clone, Debug)]
pub enum Stage<'a> {
    Input(SchemaNode, Value),
    // absolute or relative ? absolute for now
    Get {
        path: Cow<'a, [u32]>,
    },
    // absolute or relative ? absolute for now
    Set {
        destination: Cow<'a, [u32]>,
        value: ValueOrPath,
    },
    Filter {
        path: Cow<'a, [u32]>,
        condition: Condition,
    },
}

#[derive(Clone, Debug)]
pub enum ValueOrPath {
    Value(Value),
    Path(Vec<u32>),
}

#[derive(Clone, Debug)]
pub enum Condition {
    Equal {
        schema: SchemaNode,
        lhs: ValueOrPath,
        rhs: ValueOrPath,
    },
}

impl Stage<'_> {
    pub fn transform_schema<'a>(
        &'a self,
        base_schema: &'a SchemaNode,
        schema: &'a SchemaNode,
    ) -> Cow<'a, SchemaNode> {
        match self {
            Stage::Input(input_schema, _) => Cow::Borrowed(input_schema),
            Stage::Get { path } => Cow::Borrowed(base_schema.scope_ref(path.as_ref()).unwrap()),
            Stage::Set { .. } => Cow::Borrowed(schema),
            Stage::Filter { .. } => Cow::Borrowed(schema),
        }
    }

    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> std::io::Result<()> {
        let kind = match self {
            Self::Input(_, _) => 0,
            Self::Get { .. } => 1,
            Self::Set { .. } => 2,
            Self::Filter { .. } => 3,
        };

        write.write_u8(kind).await?;

        match self {
            Self::Input(schema, value) => {
                schema.write(write).await?;
                value.write(write).await?;
            }
            Self::Get { path } => write_path(path, write).await?,
            Self::Set { value, destination } => {
                write_path(destination, write).await?;
                value.write(write).await?;
            }
            Self::Filter { path, condition } => {
                write_path(path, write).await?;
                condition.write(write).await?;
            }
        }

        Ok(())
    }

    pub async fn read(
        base_schema: &SchemaNode,
        _schema: &SchemaNode,
        read: &mut (impl AsyncReadExt + Unpin),
    ) -> std::io::Result<Self> {
        Ok(match read.read_u8().await? {
            0 => {
                let schema = SchemaNode::read(read).await?;
                let value = Value::read(&schema, read).await?;
                Self::Input(schema, value)
            }
            1 => Self::Get {
                path: read_path(read).await?.into(),
            },
            2 => {
                let destination = read_path(read).await?;
                let value =
                    ValueOrPath::read(base_schema.scope_ref(&destination).unwrap(), read).await?;

                Self::Set {
                    value,
                    destination: destination.into(),
                }
            }
            3 => {
                let path = Cow::Owned(read_path(read).await?);
                let condition = Condition::read(read).await?;

                Self::Filter { path, condition }
            }
            _ => panic!(),
        })
    }
}

impl ValueOrPath {
    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> std::io::Result<()> {
        let kind = match self {
            ValueOrPath::Value(_) => 0,
            ValueOrPath::Path(_) => 1,
        };

        write.write_u8(kind).await?;

        match self {
            ValueOrPath::Value(value) => value.write(write).await?,
            ValueOrPath::Path(path) => write_path(path, write).await?,
        }

        Ok(())
    }

    pub async fn read(
        schema: &SchemaNode,
        read: &mut (impl AsyncReadExt + Unpin),
    ) -> std::io::Result<Self> {
        Ok(match read.read_u8().await? {
            0 => Self::Value(Value::read(schema, read).await?),
            1 => Self::Path(read_path(read).await?),
            _ => panic!(),
        })
    }
}

impl Condition {
    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> std::io::Result<()> {
        let kind = match self {
            Self::Equal { .. } => 0,
        };

        write.write_u8(kind).await?;

        match self {
            Condition::Equal { schema, lhs, rhs } => {
                schema.write(write).await?;
                lhs.write(write).await?;
                rhs.write(write).await?;
            }
        }

        Ok(())
    }

    pub async fn read(read: &mut (impl AsyncReadExt + Unpin)) -> std::io::Result<Self> {
        Ok(match read.read_u8().await? {
            0 => {
                let schema = SchemaNode::read(read).await?;
                let lhs = ValueOrPath::read(&schema, read).await?;
                let rhs = ValueOrPath::read(&schema, read).await?;

                Self::Equal { schema, lhs, rhs }
            }
            _ => panic!(),
        })
    }
}
