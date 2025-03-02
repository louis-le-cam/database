//! This crate is a database engine with on strict and versatile typing.
//!
//! This crate contains both the implementation of the database server and the
//! client, you can also run the database locally within you're program
//!
//! # Example
//!
//! Simple database storing an array of users
//!
//! ```
//! # use database::{Schema, Client, Server, SchemaNode, Value, Filter, StringEqual};
//! #
//! #[derive(Schema, Debug, PartialEq)]
//! enum Shape {
//!     Rectangle {
//!         width: f32,
//!         height: f32,
//!     },
//!     Triangle {
//!         a: (f32, f32),
//!         b: (f32, f32),
//!         c: (f32, f32),
//!     },
//!     Circle(f32),
//! }
//!
//! #[derive(Schema, Debug, PartialEq)]
//! struct User {
//!     name: String,
//!     favorite_shape: Option<Shape>,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let (server_stream, client_stream) = tokio::io::duplex(64);
//!
//!     let server = Server::new(SchemaNode::Unit, Value::Unit);
//!
//!     tokio::join!(
//!         server.listen(server_stream),
//!         async {
//!             let mut client = Client::<(), _>::new(client_stream)
//!                 .await?
//!                 // The `set` method actually reset the whole database including schema.
//!                 // We use it here because the database cannot yet be save on disk.
//!                 .set(vec![
//!                     User {
//!                         name: "some user 1".to_string(),
//!                         favorite_shape: Some(Shape::Circle(38.1)),
//!                     },
//!                     User {
//!                         name: "some user 2".to_string(),
//!                         favorite_shape: Some(Shape::Rectangle { width: 3.5, height: 7.0 }),
//!                     },
//!                     User {
//!                         name: "some user 3".to_string(),
//!                         favorite_shape: None,
//!                     },
//!                 ])
//!                 .await?;
//!
//!             // Get user by name
//!             assert_eq!(
//!                 client.query(|users| users.filter(|user| user.name.equal("some user 1"))).await?,
//!                 vec![User {
//!                     name: "some user 1".to_string(),
//!                     favorite_shape: Some(Shape::Circle(38.1)),
//!                 }],
//!             );
//!
//!             Ok(()) as Result<(), std::io::Error>
//!         }
//!     );
//! }
//! ```
//!
//! # See also
//! - [`Client`]
//! - [`Server`]
//! - [`Schema`]
//! - [`Expression`]
//!
//! # Protocol
//!
//! The communication protocol between the client and the server is pretty simple.
//! It can work on top of any byte stream protocol like tcp.
//!
//! There is no alignment further than byte alignment.
//!
//! Integers are stored in little-endian even though big-endian is the network
//! standard, this protocol is not network specific. This behaviour is in line
//! with the mongodb protocol as an example.
//!
//! A [`Server`] can hold multiple communications at a time. (Each call to
//! [`Server::listen`] open a new communication).
//!
//! The protocol is driven by the [`Client`] with a request/response scheme.
//!
//! A request is identified by a first byte discriminant which determine the
//! kind of request, see [`request_discriminant`].
//!
//! This discriminant is directly followed by the payload of the request
//!
//! There a three kind of requests:
//! - get schema:
//!     The request does not take any payload.
//!
//!     The request directly respond with the [`Schema`] of the database.
//! - set:
//!     The request take the new [`Schema`] then [`Value`] of the database.
//!
//!     The request does not respond anything.
//! - query:
//!     The request take an [`Expression`] as payload.
//!
//!     The request returns a [`Value`], the [`Schema`] of this value depends on the [`Expression`].
//!
//! There are three kind of data that can be sent both ways in the protocol:
//!
//! ## [`Schema`]
//! A [`Schema`] represent the shape and possible values of values, it's a type
//! in [type theory](https://en.wikipedia.org/wiki/Type_theory).
//!
//! More specifically it uses
//! [algebraic data types](https://en.wikipedia.org/wiki/Algebraic_data_type)
//! paradigm to structure the data (just like `rust` with `enum` and `struct`).
//!
//! TODO: terminal schema node, composite schema node, ...
//!
//! ## [`Value`]
//! TODO
//!
//! ## [`Expression`]
//! TODO

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
