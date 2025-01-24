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
        StringExpression, VecExpression,
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
