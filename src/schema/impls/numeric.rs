use std::{future::Future, io};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    schema_discriminant, Float32Expression, Float64Expression, Int128Expression, Int16Expression,
    Int32Expression, Int64Expression, Int8Expression, Schema, Uint128Expression, Uint16Expression,
    Uint32Expression, Uint64Expression, Uint8Expression,
};

macro_rules! impl_numerics {
    ($($name:ident $write_fn:ident $read_fn:ident $discriminant:ident $expression:ident;)*) => {
        $(
            impl Schema for $name {
                type Expression = $expression;

                fn write_schema(
                    write: &mut (impl AsyncWriteExt + Unpin + Send),
                ) -> impl Future<Output = io::Result<()>> + Send {
                    write.write_u8(schema_discriminant::$discriminant)
                }

                fn write_value(
                    &self,
                    write: &mut (impl AsyncWriteExt + Unpin + Send),
                ) -> impl Future<Output = io::Result<()>> + Send {
                    write.$write_fn(*self)
                }

                fn read_value(
                    read: &mut (impl AsyncReadExt + Unpin + Send),
                ) -> impl Future<Output = io::Result<Self>> + Send {
                    read.$read_fn()
                }
            }
        )*
    };
}

impl_numerics! {
    u8 write_u8 read_u8 UINT8 Uint8Expression;
    u16 write_u16 read_u16 UINT16 Uint16Expression;
    u32 write_u32 read_u32 UINT32 Uint32Expression;
    u64 write_u64 read_u64 UINT64 Uint64Expression;
    u128 write_u128 read_u128 UINT128 Uint128Expression;
    i8 write_i8 read_i8 INT8 Int8Expression;
    i16 write_i16 read_i16 INT16 Int16Expression;
    i32 write_i32 read_i32 INT32 Int32Expression;
    i64 write_i64 read_i64 INT64 Int64Expression;
    i128 write_i128 read_i128 INT128 Int128Expression;
    f32 write_f32 read_f32 FLOAT32 Float32Expression;
    f64 write_f64 read_f64 FLOAT64 Float64Expression;
}
