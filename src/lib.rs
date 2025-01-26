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
        expression_discriminant, BoolExpression, Equal, EqualExpression, Expression,
        ExpressionNode, Filter, FilterExpression, Float32Expression, Float64Expression, FromPath,
        Int128Expression, Int16Expression, Int32Expression, Int64Expression, Int8Expression,
        OptionExpression, Set, SetExpression, StringExpression, TupleExpression1,
        TupleExpression10, TupleExpression11, TupleExpression12, TupleExpression13,
        TupleExpression14, TupleExpression15, TupleExpression16, TupleExpression2,
        TupleExpression3, TupleExpression4, TupleExpression5, TupleExpression6, TupleExpression7,
        TupleExpression8, TupleExpression9, Uint128Expression, Uint16Expression, Uint32Expression,
        Uint64Expression, Uint8Expression, UnitExpression, VecExpression,
    },
    schema::{schema_discriminant, Schema, SchemaNode},
    value::Value,
};

pub(crate) use crate::scope::Scope;

macro_rules! io_error {
    ($kind:ident, $message:literal $(,)?) => {
        ::std::io::Error::new(
            ::std::io::ErrorKind::$kind,
            concat!(env!("CARGO_CRATE_NAME"), ": ", $message),
        )
    };
}

pub(crate) use io_error;
