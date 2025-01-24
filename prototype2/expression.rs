use std::{borrow::Cow, ops::BitAnd};

use crate::{
    schema::StringRef, BooleanMut, BooleanRef, BoxOrRef, Float32Mut, Float32Ref, Float64Mut,
    Float64Ref, Int128Mut, Int128Ref, Int16Mut, Int16Ref, Int32Mut, Int32Ref, Int64Mut, Int64Ref,
    Int8Mut, Int8Ref, OptionMut, OptionRef, Schema, SchemaNode, StringMut, Tuple2Mut, Tuple2Ref,
    Tuple3Mut, Tuple3Ref, Uint128Mut, Uint128Ref, Uint16Mut, Uint16Ref, Uint32Mut, Uint32Ref,
    Uint64Mut, Uint64Ref, Uint8Mut, Uint8Ref, UnitMut, UnitRef, Value, VecMut, VecOrSlice, VecRef,
    WithPath,
};

#[derive(Debug)]
pub enum Expression<'a> {
    // &
    Chain(BoxOrRef<'a, (Expression<'a>, Expression<'a>)>),
    // <variable ref or mut>
    Path(VecOrSlice<'a, u32>),
    // =
    Eq(BoxOrRef<'a, (Expression<'a>, Expression<'a>)>),
    // <vec>.filter
    Filter(
        BoxOrRef<
            'a,
            (
                Expression<'a>, /* target */
                Expression<'a>, /* condition */
            ),
        >,
    ),
    // <mut>.set
    Set(BoxOrRef<'a, (VecOrSlice<'a, u32>, Expression<'a>)>),
    Value(SchemaNode<'a>, Value<'a>),
}

pub trait IntoExpression {
    fn into_expression(self) -> Expression<'static>;
}

macro_rules! impl_into_expression_for_path {
    ($($name:ident $(<$($generic:ident: $contraint:path),*>)?)*) => {
        $(
            impl $(<$($generic: $contraint),*>)? IntoExpression for $name $(<$($generic),*>)? {
                fn into_expression(self) -> Expression<'static> {
                    Expression::Path(VecOrSlice::Vec(self.path().to_vec()))
                }
            }
        )*
    };
}

impl_into_expression_for_path!(
    StringRef StringMut
    BooleanRef BooleanMut
    Float64Ref Float64Mut
    Float32Ref Float32Mut
    Uint128Ref Uint128Mut
    Uint64Ref Uint64Mut
    Uint32Ref Uint32Mut
    Uint16Ref Uint16Mut
    Uint8Ref Uint8Mut
    Int128Ref Int128Mut
    Int64Ref Int64Mut
    Int32Ref Int32Mut
    Int16Ref Int16Mut
    Int8Ref Int8Mut
    UnitRef UnitMut
    OptionRef OptionMut
    VecRef VecMut
    Tuple2Ref<A: Schema, B: Schema>
    Tuple2Mut<A: Schema, B: Schema>
    Tuple3Ref<A: Schema, B: Schema, C: Schema>
    Tuple3Mut<A: Schema, B: Schema, C: Schema>
);

impl IntoExpression for &str {
    fn into_expression(self) -> Expression<'static> {
        Expression::Value(
            SchemaNode::String,
            Value::String(Cow::Owned(self.to_string())),
        )
    }
}

impl<S: Schema> IntoExpression for S {
    fn into_expression(self) -> Expression<'static> {
        Expression::Value(S::SCHEMA_NODE, self.value().into_owned())
    }
}

trait StringExpression: IntoExpression {}

impl StringExpression for &str {}
impl StringExpression for StringRef {}
impl StringExpression for StringMut {}

trait IntegerExpression: IntoExpression {}

impl IntegerExpression for Uint128Ref {}
impl IntegerExpression for Uint128Mut {}
impl IntegerExpression for Uint64Ref {}
impl IntegerExpression for Uint64Mut {}
impl IntegerExpression for Uint32Ref {}
impl IntegerExpression for Uint32Mut {}
impl IntegerExpression for Uint16Ref {}
impl IntegerExpression for Uint16Mut {}
impl IntegerExpression for Uint8Ref {}
impl IntegerExpression for Uint8Mut {}
impl IntegerExpression for Int128Ref {}
impl IntegerExpression for Int128Mut {}
impl IntegerExpression for Int64Ref {}
impl IntegerExpression for Int64Mut {}
impl IntegerExpression for Int32Ref {}
impl IntegerExpression for Int32Mut {}
impl IntegerExpression for Int16Ref {}
impl IntegerExpression for Int16Mut {}
impl IntegerExpression for Int8Ref {}
impl IntegerExpression for Int8Mut {}

impl IntegerExpression for u128 {}
impl IntegerExpression for u64 {}
impl IntegerExpression for u32 {}
impl IntegerExpression for u16 {}
impl IntegerExpression for u8 {}
impl IntegerExpression for i128 {}
impl IntegerExpression for i64 {}
impl IntegerExpression for i32 {}
impl IntegerExpression for i16 {}
impl IntegerExpression for i8 {}

pub trait StringEqual<Rhs> {
    fn equal(self, rhs: Rhs) -> Expression<'static>;
}

impl<S1: StringExpression, S2: StringExpression> StringEqual<S2> for S1 {
    fn equal(self, rhs: S2) -> Expression<'static> {
        Expression::Eq(BoxOrRef::Box(Box::new((
            self.into_expression(),
            rhs.into_expression(),
        ))))
    }
}

pub trait IntegerEqual<Rhs> {
    fn equal(self, rhs: Rhs) -> Expression<'static>;
}

impl<S1: IntegerExpression, S2: IntegerExpression> IntegerEqual<S2> for S1 {
    fn equal(self, rhs: S2) -> Expression<'static> {
        Expression::Eq(BoxOrRef::Box(Box::new((
            self.into_expression(),
            rhs.into_expression(),
        ))))
    }
}

impl BitAnd<Expression<'static>> for Expression<'static> {
    type Output = Expression<'static>;

    fn bitand(self, rhs: Expression<'static>) -> Self::Output {
        Expression::Chain(BoxOrRef::Box(Box::new((self, rhs))))
    }
}

pub trait Set<Rhs> {
    fn set(self, rhs: Rhs) -> Expression<'static>;
}

impl<S: StringExpression> Set<S> for StringMut {
    fn set(self, rhs: S) -> Expression<'static> {
        Expression::Set(BoxOrRef::Box(Box::new((
            VecOrSlice::Vec(self.path().to_vec()),
            rhs.into_expression(),
        ))))
    }
}

trait IntegerMut: WithPath {}

impl IntegerMut for Uint128Mut {}
impl IntegerMut for Uint64Mut {}
impl IntegerMut for Uint32Mut {}
impl IntegerMut for Uint16Mut {}
impl IntegerMut for Uint8Mut {}
impl IntegerMut for Int128Mut {}
impl IntegerMut for Int64Mut {}
impl IntegerMut for Int32Mut {}
impl IntegerMut for Int16Mut {}
impl IntegerMut for Int8Mut {}

impl<Lhs: IntegerMut, Rhs: IntegerExpression> Set<Rhs> for Lhs {
    fn set(self, rhs: Rhs) -> Expression<'static> {
        Expression::Set(BoxOrRef::Box(Box::new((
            VecOrSlice::Vec(self.path().to_vec()),
            rhs.into_expression(),
        ))))
    }
}
