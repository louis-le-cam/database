use std::{
    future::Future,
    io,
    num::{
        NonZero, NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8,
    },
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    io_error, schema_discriminant, Float32Expression, Float64Expression, Int128Expression,
    Int16Expression, Int32Expression, Int64Expression, Int8Expression, NonZeroInt128Expression,
    NonZeroInt16Expression, NonZeroInt32Expression, NonZeroInt64Expression, NonZeroInt8Expression,
    NonZeroUint128Expression, NonZeroUint16Expression, NonZeroUint32Expression,
    NonZeroUint64Expression, NonZeroUint8Expression, Schema, Uint128Expression, Uint16Expression,
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

macro_rules! impl_numerics_non_zero {
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
                    write.$write_fn(self.get())
                }

                fn read_value(
                    read: &mut (impl AsyncReadExt + Unpin + Send),
                ) -> impl Future<Output = io::Result<Self>> + Send {
                    async {
                        NonZero::new(read.$read_fn().await?).ok_or_else(|| io_error!(InvalidData, ""))
                    }
                }
            }
        )*
    };
}

impl_numerics_non_zero! {
    NonZeroU8 write_u8 read_u8 UINT8 NonZeroUint8Expression;
    NonZeroU16 write_u16 read_u16 UINT16 NonZeroUint16Expression;
    NonZeroU32 write_u32 read_u32 UINT32 NonZeroUint32Expression;
    NonZeroU64 write_u64 read_u64 UINT64 NonZeroUint64Expression;
    NonZeroU128 write_u128 read_u128 UINT128 NonZeroUint128Expression;
    NonZeroI8 write_i8 read_i8 INT8 NonZeroInt8Expression;
    NonZeroI16 write_i16 read_i16 INT16 NonZeroInt16Expression;
    NonZeroI32 write_i32 read_i32 INT32 NonZeroInt32Expression;
    NonZeroI64 write_i64 read_i64 INT64 NonZeroInt64Expression;
    NonZeroI128 write_i128 read_i128 INT128 NonZeroInt128Expression;
}
