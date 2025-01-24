use std::borrow::Cow;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{pipeline::Stage, schema::SchemaNode};

pub enum Request<'a> {
    GetSchema,
    Get { path: Cow<'a, [u32]> },
    // TODO: use feature flag or similar,
    // there are different because parsing
    // a stage need the last stage schema
    Pipeline(Cow<'a, [Stage<'a>]>),
}

impl Request<'_> {
    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> std::io::Result<()> {
        let kind = match self {
            Self::GetSchema => 0,
            Self::Get { .. } => 1,
            Self::Pipeline(_) => 2,
        };

        write.write_u8(kind).await?;

        match self {
            Self::GetSchema => {}
            Self::Get { path } => {
                write.write_u32(path.len() as u32).await?;
                for segment in path.iter() {
                    write.write_u32(*segment).await?;
                }
            }
            Self::Pipeline(stages) => {
                write.write_u32(stages.len() as u32).await?;
                for stage in stages.iter() {
                    stage.write(write).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn read(
        base_schema: &SchemaNode,
        read: &mut (impl AsyncReadExt + Unpin),
    ) -> std::io::Result<Self> {
        Ok(match read.read_u8().await? {
            0 => Self::GetSchema,
            1 => {
                let length = read.read_u32().await?;
                let mut path = Vec::with_capacity(length as usize);

                for _ in 0..length {
                    path.push(read.read_u32().await?);
                }

                Self::Get {
                    path: Cow::Owned(path),
                }
            }
            2 => {
                let length = read.read_u32().await?;
                let mut stages = Vec::with_capacity(length as usize);

                let mut schema = Cow::Borrowed(base_schema);

                for _ in 0..length {
                    let stage = Stage::read(base_schema, schema.as_ref(), read).await?;
                    // TODO: do not into_owned here
                    schema = Cow::Owned(
                        stage
                            .transform_schema(base_schema, schema.as_ref())
                            .into_owned(),
                    );
                    stages.push(stage);
                }

                Self::Pipeline(Cow::Owned(stages))
            }
            _ => panic!(),
        })
    }
}
