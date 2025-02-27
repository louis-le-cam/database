use std::{future::Future, io};

use tokio::io::AsyncWriteExt;

use crate::{
    expression_discriminant, io_error, schema_discriminant, Expression, Schema, TupleExpression1,
    TupleExpression10, TupleExpression11, TupleExpression12, TupleExpression13, TupleExpression14,
    TupleExpression15, TupleExpression16, TupleExpression2, TupleExpression3, TupleExpression4,
    TupleExpression5, TupleExpression6, TupleExpression7, TupleExpression8, TupleExpression9,
};

macro_rules! generate {
    ($($name:ident $($field:ident)*;)*) => {
        $(
            impl<$($field: Schema + Send + Sync,)*> Schema for ($($field,)*) {
                type Expression = $name<$($field,)*>;

                fn write_schema(
                    write: &mut (impl AsyncWriteExt + Unpin + Send),
                ) -> impl Future<Output = io::Result<()>> + Send {
                    async {
                        write.write_u8(schema_discriminant::PRODUCT).await?;

                        #[allow(non_snake_case)]
                        {
                            $(let $field = ();)*
                            write.write_u32(0 $( + (1, $field).0)*).await?;
                        }

                        $($field::write_schema(write).await?;)*

                        Ok(())
                    }
                }

                fn write_value(
                    &self,
                    write: &mut (impl AsyncWriteExt + Unpin + Send),
                ) -> impl Future<Output = io::Result<()>> + Send {
                    async {
                        #[allow(non_snake_case)]
                        let ($($field,)*) = self;

                        $($field.write_value(write).await?;)*

                        Ok(())
                    }
                }

                fn read_value(
                    read: &mut (impl tokio::io::AsyncReadExt + Unpin + Send),
                ) -> impl Future<Output = io::Result<Self>> + Send {
                    async {
                        Ok(($($field::read_value(read).await?,)*))
                    }
                }
            }

            impl<$($field: Expression,)*> Expression for ($($field,)*) where $($field::Target: Send + Sync,)* {
                type Target = ($($field::Target,)*);

                fn write(
                    self,
                    write: &mut (impl AsyncWriteExt + Unpin + Send),
                ) -> impl Future<Output = io::Result<()>> {
                    async move {
                        write.write_u8(expression_discriminant::PRODUCT).await?;

                        #[allow(non_snake_case)]
                        {
                            $(let $field = ();)*
                            write.write_u32(0 $( + (1, $field).0)*).await?;
                        }

                        #[allow(non_snake_case)]
                        {
                            let ($($field,)*) = self;
                            $($field.write(write).await?;)*
                        }

                        Ok(())
                    }
                }
            }
        )*
    };
}

generate!(
    TupleExpression1 A;
    TupleExpression2 A B;
    TupleExpression3 A B C;
    TupleExpression4 A B C D;
    TupleExpression5 A B C D E;
    TupleExpression6 A B C D E F;
    TupleExpression7 A B C D E F G;
    TupleExpression8 A B C D E F G H;
    TupleExpression9 A B C D E F G H I;
    TupleExpression10 A B C D E F G H I J;
    TupleExpression11 A B C D E F G H I J K;
    TupleExpression12 A B C D E F G H I J K L;
    TupleExpression13 A B C D E F G H I J K L M;
    TupleExpression14 A B C D E F G H I J K L M N;
    TupleExpression15 A B C D E F G H I J K L M N O;
    TupleExpression16 A B C D E F G H I J K L M N O P;
);
