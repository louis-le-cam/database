use std::io;

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, io_error, Expression, FromPath};

macro_rules! impl_numerics {
    ($($name:ident $expression:ident;)*) => {
        $(
            #[derive(Clone)]
            pub struct $expression(Vec<u32>);

            impl Expression for $expression {
                type Target = $name;

                async fn write(self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
                    write.write_u8(expression_discriminant::PATH).await?;
                    write
                        .write_u32(self.0.len().try_into().map_err(|_| {
                            io_error!(
                                OutOfMemory,
                                "path expression length doesn't fit into a 32 bit unsigned integer",
                            )
                        })?)
                        .await?;

                    for segment in &self.0 {
                        write.write_u32(*segment).await?;
                    }

                    Ok(())
                }
            }

            impl FromPath for $expression {
                fn from_path(path: Vec<u32>) -> Self {
                    Self(path)
                }
            }
        )*
    };
}

impl_numerics! {
    u8 Uint8Expression;
    u16 Uint16Expression;
    u32 Uint32Expression;
    u64 Uint64Expression;
    u128 Uint128Expression;
    i8 Int8Expression;
    i16 Int16Expression;
    i32 Int32Expression;
    i64 Int64Expression;
    i128 Int128Expression;
    f32 Float32Expression;
    f64 Float64Expression;
}
