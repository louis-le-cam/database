mod client;
mod expression;
mod schema;
mod scope;
mod server;
mod value;

pub use crate::{
    client::Client,
    expression::{
        expression_discriminant, And, AndExpression, BoolOperators, Chain, ChainExpression,
        ConditionExpression, EqualExpression, Expression, ExpressionNode, Filter, FilterExpression,
        FromPath, FuseExpression, GetExpression, Int128Equal, Int16Equal, Int32Equal, Int64Equal,
        Int8Equal, MapVariantExpression, MapVec, OptionOperators, PathExpression, Set,
        SetExpression, StringEqual, TupleExpression1, TupleExpression10, TupleExpression11,
        TupleExpression12, TupleExpression13, TupleExpression14, TupleExpression15,
        TupleExpression16, TupleExpression2, TupleExpression3, TupleExpression4, TupleExpression5,
        TupleExpression6, TupleExpression7, TupleExpression8, TupleExpression9, Uint128Equal,
        Uint16Equal, Uint32Equal, Uint64Equal, Uint8Equal, VecGet,
    },
    schema::{schema_discriminant, DefaultKey, Key, OptionMapped, Schema, SchemaNode, SlotMap},
    server::{request_discriminant, Server},
    value::Value,
};

pub use database_derive::Schema;

pub(crate) use crate::scope::Scope;

#[doc(hidden)]
pub mod __internal {
    pub use tokio;
}

macro_rules! io_error {
    ($kind:ident, $message:literal $(,)?) => {
        ::std::io::Error::new(
            ::std::io::ErrorKind::$kind,
            concat!(env!("CARGO_CRATE_NAME"), ": ", $message),
        )
    };
}

pub(crate) use io_error;
