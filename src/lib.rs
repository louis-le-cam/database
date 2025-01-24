mod client;
mod database;
mod expression;
mod schema;
mod value;

pub use crate::{
    client::Client,
    database::Database,
    expression::{
        BoolExpression, Equal, EqualExpression, Expression, ExpressionNode, FromPath,
        StringExpression, TupleExpression1, TupleExpression10, TupleExpression11,
        TupleExpression12, TupleExpression13, TupleExpression14, TupleExpression15,
        TupleExpression16, TupleExpression2, TupleExpression3, TupleExpression4, TupleExpression5,
        TupleExpression6, TupleExpression7, TupleExpression8, TupleExpression9, UnitExpression,
        VecExpression,
    },
    schema::{Schema, SchemaLeaf, SchemaNode},
    value::{Value, ValueLeaf},
};

macro_rules! io_error {
    ($kind:ident, $message:literal $(,)?) => {
        ::std::io::Error::new(
            ::std::io::ErrorKind::$kind,
            concat!(env!("CARGO_CRATE_NAME"), ": ", $message),
        )
    };
}

pub(crate) use io_error;
