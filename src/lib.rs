mod client;
mod database;
mod expression;
mod schema;
mod scope;
mod value;

pub use crate::{
    client::Client,
    database::Database,
    expression::{
        expression_discriminant, And, AndExpression, BoolExpression, Chain, ChainExpression,
        EqualExpression, Expression, ExpressionNode, Filter, FilterExpression, Float32Expression,
        Float64Expression, FromPath, Int128Equal, Int128Expression, Int16Equal, Int16Expression,
        Int32Equal, Int32Expression, Int64Equal, Int64Expression, Int8Equal, Int8Expression,
        OptionExpression, Set, SetExpression, StringEqual, StringExpression, TupleExpression1,
        TupleExpression10, TupleExpression11, TupleExpression12, TupleExpression13,
        TupleExpression14, TupleExpression15, TupleExpression16, TupleExpression2,
        TupleExpression3, TupleExpression4, TupleExpression5, TupleExpression6, TupleExpression7,
        TupleExpression8, TupleExpression9, Uint128Equal, Uint128Expression, Uint16Equal,
        Uint16Expression, Uint32Equal, Uint32Expression, Uint64Equal, Uint64Expression, Uint8Equal,
        Uint8Expression, UnitExpression, VecExpression,
    },
    schema::{schema_discriminant, Schema, SchemaNode},
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
