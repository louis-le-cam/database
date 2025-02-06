mod expression_read;
mod expression_write;
mod request_read;
mod request_write;
mod schema_read;
mod schema_write;
mod value_read;
mod value_write;

pub use self::{
    expression_read::{ExpressionRead, ExpressionReadResult},
    expression_write::ExpressionWrite,
    request_read::RequestRead,
    request_write::RequestWrite,
    schema_read::{SchemaRead, SchemaReadResult, SchemasRead},
    schema_write::{SchemaWrite, SchemasWrite},
    value_read::{ValueRead, ValuesRead},
    value_write::{ValueWrite, ValuesWrite},
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub trait FromStream<S: AsyncWriteExt + AsyncReadExt + Unpin> {
    fn from_stream(stream: S) -> Self;
}

mod request_discriminant {
    pub const GET_SCHEMA: u8 = 0;
    pub const SET_SCHEMA: u8 = 1;
    pub const QUERY: u8 = 2;
}

mod schema_discriminant {
    pub const PRODUCT: u8 = 0;
    pub const SUM: u8 = 1;
    pub const LIST: u8 = 2;
    pub const STRING: u8 = 3;
    pub const BOOLEAN: u8 = 4;
    pub const UNIT: u8 = 5;
    pub const UINT8: u8 = 6;
    pub const UINT16: u8 = 7;
    pub const UINT32: u8 = 8;
    pub const UINT64: u8 = 9;
    pub const UINT128: u8 = 10;
    pub const INT8: u8 = 11;
    pub const INT16: u8 = 12;
    pub const INT32: u8 = 13;
    pub const INT64: u8 = 14;
    pub const INT128: u8 = 15;
    pub const FLOAT32: u8 = 16;
    pub const FLOAT64: u8 = 17;
}

mod expression_discriminant {
    pub const PATH: u8 = 0;
    pub const VALUE: u8 = 1;
    pub const SET: u8 = 2;
    pub const EQUAL: u8 = 3;
    pub const FILTER: u8 = 4;
    pub const AND: u8 = 5;
    pub const MAP_VARIANT: u8 = 6;
    pub const CHAIN: u8 = 7;
}
