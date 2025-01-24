use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{SchemaNode, Value};

pub enum Request<'a> {
    GetSchema,
    SetSchema(SchemaNode<'a>, Value<'a>),
    Get,
}

impl Request<'_> {
    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        let kind = match self {
            Request::GetSchema => 0,
            Request::SetSchema(_, _) => 1,
            Request::Get => 2,
        };

        write.write_u8(kind).await?;

        match self {
            Request::SetSchema(schema, value) => {
                schema.write(write).await?;
                value.write(write).await?;
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn read(read: &mut (impl AsyncReadExt + Unpin)) -> io::Result<Option<Self>> {
        match read.read_u8().await {
            Ok(0) => Ok(Some(Self::GetSchema)),
            Ok(1) => {
                let schema = SchemaNode::read(read).await?;
                let value = Value::read(&schema, read).await?;
                Ok(Some(Self::SetSchema(schema, value)))
            }
            Ok(2) => Ok(Some(Self::Get)),
            Ok(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid request kind",
            )),
            Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(err) => Err(err),
        }
    }
}
