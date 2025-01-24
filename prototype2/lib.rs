mod client;
mod expression;
mod request;
mod schema;
mod server;
mod test;
mod value;

use std::{mem::ManuallyDrop, ops::Deref};

pub use crate::{
    client::Client,
    expression::{Expression, IntegerEqual, IntoExpression, Set, StringEqual},
    request::Request,
    schema::{
        BooleanMut, BooleanRef, Float32Mut, Float32Ref, Float64Mut, Float64Ref, Int128Mut,
        Int128Ref, Int16Mut, Int16Ref, Int32Mut, Int32Ref, Int64Mut, Int64Ref, Int8Mut, Int8Ref,
        OptionMut, OptionRef, Schema, SchemaNode, SchemaOwnedConst, StringMut, StringRef,
        Tuple2Mut, Tuple2Ref, Tuple3Mut, Tuple3Ref, Uint128Mut, Uint128Ref, Uint16Mut, Uint16Ref,
        Uint32Mut, Uint32Ref, Uint64Mut, Uint64Ref, Uint8Mut, Uint8Ref, UnitMut, UnitRef, VecMut,
        VecRef, WithPath,
    },
    server::Server,
    value::Value,
};

#[derive(Debug, Clone)]
pub enum BoxOrRef<'a, T> {
    Box(Box<T>),
    Ref(&'a T),
    /// Should only be used in const contexts, it's a workaround compiler limitations
    Const(&'a ManuallyDrop<T>),
}

#[derive(Debug, Clone)]
pub enum VecOrSlice<'a, T: Clone> {
    Vec(Vec<T>),
    Slice(&'a [T]),
    /// Should only be used in const contexts, it's a workaround compiler limitations
    Const(&'a ManuallyDrop<[T]>),
}

impl<'a, T: Clone> Deref for VecOrSlice<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match self {
            VecOrSlice::Vec(vec) => vec.as_slice(),
            VecOrSlice::Slice(slice) => slice,
            VecOrSlice::Const(slice) => slice,
        }
    }
}

impl<'a, T> Deref for BoxOrRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            BoxOrRef::Box(boxed) => boxed.as_ref(),
            BoxOrRef::Ref(reference) => reference,
            BoxOrRef::Const(reference) => reference,
        }
    }
}
