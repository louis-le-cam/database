use std::io;

use tokio::io::AsyncWriteExt;

use crate::{io_error, Expression, FromPath, Schema};

macro_rules! generate {
    ($($last_index:tt $name:ident $($field:ident)*;)*) => {
        $(
            pub struct $name<$($field: Schema + Send + Sync,)*>($(pub $field::Expression,)* Vec<u32>);

            impl<$($field: Schema + Send + Sync,)*> Expression for $name<$($field,)*> {
                type Target = ($($field,)*);

                async fn write(self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
                    write.write_u8(0).await?;
                    write
                        .write_u32(self.$last_index.len().try_into().map_err(|_| {
                            io_error!(
                                OutOfMemory,
                                "path expression length doesn't fit into a 32 bit unsigned integer",
                            )
                        })?)
                        .await?;

                    for segment in &self.$last_index {
                        write.write_u32(*segment).await?;
                    }

                    Ok(())
                }
            }

            impl<$($field: Schema + Send + Sync,)*> FromPath for $name<$($field,)*> {
                fn from_path(path: Vec<u32>) -> Self {
                    let mut i = 0;
                    #[allow(unused_assignments)]
                    Self($($field::Expression::from_path((path.iter().copied().chain([i]).collect(), i += 1).0)),*, path)
                }
            }
        )*
    };
}

generate!(
    1 TupleExpression1 A;
    2 TupleExpression2 A B;
    3 TupleExpression3 A B C;
    4 TupleExpression4 A B C D;
    5 TupleExpression5 A B C D E;
    6 TupleExpression6 A B C D E F;
    7 TupleExpression7 A B C D E F G;
    8 TupleExpression8 A B C D E F G H;
    9 TupleExpression9 A B C D E F G H I;
    10 TupleExpression10 A B C D E F G H I J;
    11 TupleExpression11 A B C D E F G H I J K;
    12 TupleExpression12 A B C D E F G H I J K L;
    13 TupleExpression13 A B C D E F G H I J K L M;
    14 TupleExpression14 A B C D E F G H I J K L M N;
    15 TupleExpression15 A B C D E F G H I J K L M N O;
    16 TupleExpression16 A B C D E F G H I J K L M N O P;
);
