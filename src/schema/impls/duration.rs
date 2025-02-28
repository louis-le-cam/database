use std::{future::Future, io, time::Duration};

use tokio::io::AsyncWriteExt;

use crate::{io_error, schema_discriminant, PathExpression, Schema};

impl Schema for Duration {
    type Expression = PathExpression<Duration>;

    fn write_schema(
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        async {
            write.write_u8(schema_discriminant::PRODUCT).await?;
            write.write_u32(2).await?;
            u64::write_schema(write).await?;
            u32::write_schema(write).await?;

            Ok(())
        }
    }

    fn write_value(
        &self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        async move {
            write.write_u64(self.as_secs()).await?;
            write.write_u32(self.subsec_nanos()).await?;

            Ok(())
        }
    }

    fn read_value(
        read: &mut (impl tokio::io::AsyncReadExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<Self>> + Send {
        async {
            let seconds = read.read_u64().await?;
            let subsec_nanos = read.read_u32().await?;

            if subsec_nanos >= 1_000_000_000 {
                return Err(io_error!(
                    InvalidData,
                    "sub-seconds nanoseconds in duration are greater or equal than 1_000_000_000"
                ));
            }

            Ok(Duration::new(seconds, subsec_nanos))
        }
    }
}
