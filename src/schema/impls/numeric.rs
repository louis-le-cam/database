use std::{
    future::Future,
    io,
    num::{
        NonZero, NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8,
    },
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{io_error, schema_discriminant, PathExpression, Schema};

macro_rules! impl_numerics {
    ($($name:ident $write_fn:ident $read_fn:ident $discriminant:ident;)*) => {
        $(
            impl Schema for $name {
                type Expression = PathExpression<$name>;

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
    u8 write_u8 read_u8 UINT8;
    u16 write_u16 read_u16 UINT16;
    u32 write_u32 read_u32 UINT32;
    u64 write_u64 read_u64 UINT64;
    u128 write_u128 read_u128 UINT128;
    i8 write_i8 read_i8 INT8;
    i16 write_i16 read_i16 INT16;
    i32 write_i32 read_i32 INT32;
    i64 write_i64 read_i64 INT64;
    i128 write_i128 read_i128 INT128;
    f32 write_f32 read_f32 FLOAT32;
    f64 write_f64 read_f64 FLOAT64;
}

macro_rules! impl_numerics_non_zero {
    ($($name:ident $write_fn:ident $read_fn:ident $discriminant:ident;)*) => {
        $(
            impl Schema for $name {
                type Expression = PathExpression<$name>;

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
    NonZeroU8 write_u8 read_u8 UINT8;
    NonZeroU16 write_u16 read_u16 UINT16;
    NonZeroU32 write_u32 read_u32 UINT32;
    NonZeroU64 write_u64 read_u64 UINT64;
    NonZeroU128 write_u128 read_u128 UINT128;
    NonZeroI8 write_i8 read_i8 INT8;
    NonZeroI16 write_i16 read_i16 INT16;
    NonZeroI32 write_i32 read_i32 INT32;
    NonZeroI64 write_i64 read_i64 INT64;
    NonZeroI128 write_i128 read_i128 INT128;
}
