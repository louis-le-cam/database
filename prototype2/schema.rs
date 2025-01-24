use std::{borrow::Cow, io, mem::ManuallyDrop};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{BoxOrRef, Value, VecOrSlice};

#[macro_export(local_inner_macros)]
macro_rules! schema {
    ($vis:vis struct $name:ident { $field_vis:vis $($field:ident: $type:ty),* $(,)? } $($tail:tt)*) => {
        $vis struct $name {
            $($field_vis $field: $type,)*
        }

        impl $crate::SchemaOwnedConst<::core::mem::ManuallyDrop<[(
            ::std::borrow::Cow<'static, str>,
            $crate::SchemaNode<'static>,
        ); 0 $(+ (1, ::core::stringify!($field)).0)*]>> for $name {
            const SCHEMA_OWNED_CONST: ::core::mem::ManuallyDrop<[(
                ::std::borrow::Cow<'static, str>,
                $crate::SchemaNode<'static>
            ); 0 $(+ (1, ::core::stringify!($field)).0)*]> = ::core::mem::ManuallyDrop::new([
                $((::std::borrow::Cow::Borrowed(::core::stringify!($field)), <$type as $crate::Schema>::SCHEMA_NODE),)*
            ]);
        }

        impl $crate::Schema for $name {
            const SCHEMA_NODE: $crate::SchemaNode<'static> =
                $crate::SchemaNode::Product($crate::VecOrSlice::Const(
                    &<$name as $crate::SchemaOwnedConst<::core::mem::ManuallyDrop<[(
                        ::std::borrow::Cow<'static, str>,
                        $crate::SchemaNode<'static>,
                    ); 0 $(+ (1, ::core::stringify!($field)).0)*]>>>::SCHEMA_OWNED_CONST
                ));

            fn value<'a>(&'a self) -> $crate::Value<'a> {
                // TODO: avoid allocation here
                $crate::Value::Product(::std::borrow::Cow::Owned(::std::vec![
                    $($crate::Schema::value(&self.$field),)*
                ]))
            }

            ::paste::paste! {
                type Ref = [<$name Ref>];
                type Mut = [<$name Mut>];
            }
        }

        ::paste::paste! {
            #[allow(dead_code)]
            $vis struct [<$name Ref>] {
                $($field_vis $field: <$type as $crate::Schema>::Ref,)*
            }

            #[allow(dead_code)]
            $vis struct [<$name Mut>] {
                $($field_vis $field: <$type as $crate::Schema>::Mut,)*
            }

            #[allow(unused_assignments)]
            impl $crate::WithPath for [<$name Ref>] {
                fn from_path(path: ::std::borrow::Cow<'_, [u32]>) -> Self {
                    let mut i = 0;
                    Self {
                        $($field: <<$type as $crate::Schema>::Ref as $crate::WithPath>::from_path(path.iter().copied().chain([(i, i += 1).0]).collect()),)*
                    }
                }

                fn path(&self) -> &[u32] {
                    ($(self.$field.path(),)*).0.split_last().unwrap().1
                }
            }

            #[allow(unused_assignments)]
            impl $crate::WithPath for [<$name Mut>] {
                fn from_path(path: ::std::borrow::Cow<'_, [u32]>) -> Self {
                    let mut i = 0;
                    Self {
                        $($field: <<$type as $crate::Schema>::Mut as $crate::WithPath>::from_path(path.iter().copied().chain([(i, i += 1).0]).collect()),)*
                    }
                }

                fn path(&self) -> &[u32] {
                    ($(self.$field.path(),)*).0.split_last().unwrap().1
                }
            }
        }


        $crate::schema!($($tail)*);
    };

    () => {};
}

#[derive(Debug, Clone)]
pub enum SchemaNode<'a> {
    Product(VecOrSlice<'a, (Cow<'a, str>, SchemaNode<'a>)>),
    Sum(VecOrSlice<'a, (Cow<'a, str>, SchemaNode<'a>)>),
    List(BoxOrRef<'a, SchemaNode<'a>>),
    String,
    Boolean,
    Float64,
    Float32,
    Uint128,
    Uint64,
    Uint32,
    Uint16,
    Uint8,
    Int128,
    Int64,
    Int32,
    Int16,
    Int8,
    Unit,
}

impl SchemaNode<'_> {
    pub async fn write(&self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
        let mut nodes: Vec<(Option<&str>, &SchemaNode<'_>)> = Vec::from([(None, self)]);

        while let Some((name, node)) = nodes.last() {
            if let Some(name) = name {
                write.write_u64(name.len() as u64).await?;
                write.write_all(name.as_bytes()).await?;
            }

            let kind = match node {
                SchemaNode::Product(_) => 0,
                SchemaNode::Sum(_) => 1,
                SchemaNode::List(_) => 2,
                SchemaNode::String => 3,
                SchemaNode::Boolean => 4,
                SchemaNode::Float64 => 5,
                SchemaNode::Float32 => 6,
                SchemaNode::Uint128 => 7,
                SchemaNode::Uint64 => 8,
                SchemaNode::Uint32 => 9,
                SchemaNode::Uint16 => 10,
                SchemaNode::Uint8 => 11,
                SchemaNode::Int128 => 12,
                SchemaNode::Int64 => 13,
                SchemaNode::Int32 => 14,
                SchemaNode::Int16 => 15,
                SchemaNode::Int8 => 16,
                SchemaNode::Unit => 17,
            };

            write.write_u8(kind).await?;

            match node {
                SchemaNode::Product(fields) => {
                    write
                        .write_u32(fields.len().try_into().map_err(|_| {
                            std::io::Error::new(
                                io::ErrorKind::InvalidData,
                                "product schema with more than 2^32 - 1 fields",
                            )
                        })?)
                        .await?;

                    nodes
                        .splice(
                            nodes.len() - 1..nodes.len(),
                            fields
                                .iter()
                                .rev()
                                .map(|(name, node)| (Some(name.as_ref()), node)),
                        )
                        .for_each(drop);
                }
                SchemaNode::Sum(variants) => {
                    write
                        .write_u32(variants.len().try_into().map_err(|_| {
                            std::io::Error::new(
                                io::ErrorKind::InvalidData,
                                "sum schema with more than 2^32 - 1 variants",
                            )
                        })?)
                        .await?;

                    nodes
                        .splice(
                            nodes.len() - 1..nodes.len(),
                            variants
                                .iter()
                                .rev()
                                .map(|(name, node)| (Some(name.as_ref()), node)),
                        )
                        .for_each(drop);
                }
                SchemaNode::List(inner) => {
                    *nodes.last_mut().unwrap() = (None, inner);
                }
                _ => {
                    nodes.pop();
                }
            }
        }

        Ok(())
    }

    pub async fn read(read: &mut (impl AsyncReadExt + Unpin)) -> io::Result<Self> {
        match read.read_u8().await? {
            0 => {
                let length = read.read_u32().await?;
                let mut fields = Vec::with_capacity(length.try_into().unwrap());

                for _ in 0..length {
                    let name_length = read.read_u64().await?;
                    let mut name_bytes = vec![0; name_length.try_into().unwrap()];
                    read.read_exact(&mut name_bytes).await?;

                    fields.push((
                        Cow::Owned(String::from_utf8(name_bytes).map_err(|_| {
                            io::Error::new(io::ErrorKind::InvalidData, "non utf8 field name")
                        })?),
                        Box::pin(SchemaNode::read(read)).await?,
                    ));
                }

                Ok(SchemaNode::Product(VecOrSlice::Vec(fields)))
            }
            1 => {
                let length = read.read_u32().await?;

                let mut variants = Vec::with_capacity(length.try_into().unwrap());

                for _ in 0..length {
                    let name_length = read.read_u64().await?;
                    let mut name_bytes = vec![0; name_length.try_into().unwrap()];
                    read.read_exact(&mut name_bytes).await?;

                    variants.push((
                        Cow::Owned(String::from_utf8(name_bytes).map_err(|_| {
                            io::Error::new(io::ErrorKind::InvalidData, "non utf8 variant name")
                        })?),
                        Box::pin(SchemaNode::read(read)).await?,
                    ));
                }

                Ok(SchemaNode::Sum(VecOrSlice::Vec(variants)))
            }
            2 => Ok(SchemaNode::List(BoxOrRef::Box(Box::new(
                Box::pin(SchemaNode::read(read)).await?,
            )))),
            3 => Ok(SchemaNode::String),
            4 => Ok(SchemaNode::Boolean),
            5 => Ok(SchemaNode::Float64),
            6 => Ok(SchemaNode::Float32),
            7 => Ok(SchemaNode::Uint128),
            8 => Ok(SchemaNode::Uint64),
            9 => Ok(SchemaNode::Uint32),
            10 => Ok(SchemaNode::Uint16),
            11 => Ok(SchemaNode::Uint8),
            12 => Ok(SchemaNode::Int128),
            13 => Ok(SchemaNode::Int64),
            14 => Ok(SchemaNode::Int32),
            15 => Ok(SchemaNode::Int16),
            16 => Ok(SchemaNode::Int8),
            17 => Ok(SchemaNode::Unit),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid request kind",
            )),
        }
    }
}

impl PartialEq for SchemaNode<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SchemaNode::Product(lhs), SchemaNode::Product(rhs)) => lhs.iter().eq(rhs.iter()),
            (SchemaNode::Sum(lhs), SchemaNode::Sum(rhs)) => lhs.iter().eq(rhs.iter()),
            (SchemaNode::List(lhs), SchemaNode::List(rhs)) => **lhs == **rhs,
            (SchemaNode::String, SchemaNode::String)
            | (SchemaNode::Boolean, SchemaNode::Boolean)
            | (SchemaNode::Float64, SchemaNode::Float64)
            | (SchemaNode::Float32, SchemaNode::Float32)
            | (SchemaNode::Uint128, SchemaNode::Uint128)
            | (SchemaNode::Uint64, SchemaNode::Uint64)
            | (SchemaNode::Uint32, SchemaNode::Uint32)
            | (SchemaNode::Uint16, SchemaNode::Uint16)
            | (SchemaNode::Uint8, SchemaNode::Uint8)
            | (SchemaNode::Int128, SchemaNode::Int128)
            | (SchemaNode::Int64, SchemaNode::Int64)
            | (SchemaNode::Int32, SchemaNode::Int32)
            | (SchemaNode::Int16, SchemaNode::Int16)
            | (SchemaNode::Int8, SchemaNode::Int8)
            | (SchemaNode::Unit, SchemaNode::Unit) => true,
            _ => false,
        }
    }
}

macro_rules! generate_simple_ref_mut {
    ($($ref:ident $mut:ident;)*) => {
        $(
            pub struct $ref(Vec<u32>);

            pub struct $mut($ref);

            impl WithPath for $ref {
                fn from_path(path: Cow<'_, [u32]>) -> Self {
                    Self(path.into_owned())
                }

                fn path(&self) -> &[u32] {
                    &self.0
                }
            }

            impl WithPath for $mut {
                fn from_path(path: Cow<'_, [u32]>) -> Self {
                    Self($ref(path.into_owned()))
                }

                fn path(&self) -> &[u32] {
                    &self.0.0
                }
            }
        )*
    };
}

generate_simple_ref_mut!(
    StringRef StringMut;
    BooleanRef BooleanMut;
    Float64Ref Float64Mut;
    Float32Ref Float32Mut;
    Uint128Ref Uint128Mut;
    Uint64Ref Uint64Mut;
    Uint32Ref Uint32Mut;
    Uint16Ref Uint16Mut;
    Uint8Ref Uint8Mut;
    Int128Ref Int128Mut;
    Int64Ref Int64Mut;
    Int32Ref Int32Mut;
    Int16Ref Int16Mut;
    Int8Ref Int8Mut;
    UnitRef UnitMut;
    OptionRef OptionMut;
    VecRef VecMut;
);

pub trait Schema {
    const SCHEMA_NODE: SchemaNode<'static>;

    fn value<'a>(&'a self) -> Value<'a>;

    type Ref: WithPath;
    type Mut: WithPath;
}

pub trait WithPath {
    fn from_path(path: Cow<'_, [u32]>) -> Self;

    fn path(&self) -> &[u32];
}

impl Schema for String {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::String;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::String(Cow::Borrowed(self))
    }

    type Ref = StringRef;
    type Mut = StringMut;
}

impl Schema for bool {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Boolean;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Boolean(*self)
    }

    type Ref = BooleanRef;
    type Mut = BooleanMut;
}

impl Schema for f64 {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Float64;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Float64(*self)
    }

    type Ref = Float64Ref;
    type Mut = Float64Mut;
}

impl Schema for f32 {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Float32;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Float32(*self)
    }

    type Ref = Float32Ref;
    type Mut = Float32Mut;
}

impl Schema for u128 {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Uint128;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Uint128(*self)
    }

    type Ref = Uint128Ref;
    type Mut = Uint128Mut;
}

impl Schema for u64 {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Uint64;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Uint64(*self)
    }

    type Ref = Uint64Ref;
    type Mut = Uint64Mut;
}

impl Schema for u32 {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Uint32;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Uint32(*self)
    }

    type Ref = Uint32Ref;
    type Mut = Uint32Mut;
}

impl Schema for u16 {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Uint16;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Uint16(*self)
    }

    type Ref = Uint16Ref;
    type Mut = Uint16Mut;
}

impl Schema for u8 {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Uint8;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Uint8(*self)
    }

    type Ref = Uint8Ref;
    type Mut = Uint8Mut;
}
impl Schema for i128 {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Int128;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Int128(*self)
    }

    type Ref = Int128Ref;
    type Mut = Int128Mut;
}

impl Schema for i64 {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Int64;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Int64(*self)
    }

    type Ref = Int64Ref;
    type Mut = Int64Mut;
}

impl Schema for i32 {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Int32;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Int32(*self)
    }

    type Ref = Int32Ref;
    type Mut = Int32Mut;
}

impl Schema for i16 {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Int16;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Int16(*self)
    }

    type Ref = Int16Ref;
    type Mut = Int16Mut;
}

impl Schema for i8 {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Int8;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Int8(*self)
    }

    type Ref = Int8Ref;
    type Mut = Int8Mut;
}

impl Schema for () {
    const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Unit;

    fn value<'a>(&'a self) -> Value<'a> {
        Value::Unit
    }

    type Ref = UnitRef;
    type Mut = UnitMut;
}

/// Used to workaround compiler limitations
pub trait SchemaOwnedConst<T> {
    const SCHEMA_OWNED_CONST: T;
}

impl<T: Schema> SchemaOwnedConst<ManuallyDrop<[(Cow<'static, str>, SchemaNode<'static>); 2]>>
    for Option<T>
{
    const SCHEMA_OWNED_CONST: ManuallyDrop<[(Cow<'static, str>, SchemaNode<'static>); 2]> =
        ManuallyDrop::new([
            (Cow::Borrowed("None"), SchemaNode::Unit),
            (Cow::Borrowed("Some"), T::SCHEMA_NODE),
        ]);
}

impl<T: Schema> Schema for Option<T> {
    const SCHEMA_NODE: SchemaNode<'static> =
        SchemaNode::Sum(VecOrSlice::Const(&Self::SCHEMA_OWNED_CONST));

    fn value<'a>(&'a self) -> Value<'a> {
        match self {
            None => Value::Sum(0, BoxOrRef::Ref(&Value::Unit)),
            // TODO: avoid box allocation
            Some(inner) => Value::Sum(1, BoxOrRef::Box(Box::new(inner.value()))),
        }
    }

    type Ref = OptionRef;
    type Mut = OptionMut;
}

impl<T: Schema> SchemaOwnedConst<ManuallyDrop<SchemaNode<'static>>> for Vec<T> {
    const SCHEMA_OWNED_CONST: ManuallyDrop<SchemaNode<'static>> = ManuallyDrop::new(T::SCHEMA_NODE);
}

impl<T: Schema> Schema for Vec<T> {
    const SCHEMA_NODE: SchemaNode<'static> =
        SchemaNode::List(BoxOrRef::Const(&Self::SCHEMA_OWNED_CONST));

    fn value<'a>(&'a self) -> Value<'a> {
        Value::List(Cow::Owned(
            self.iter().map(|element| element.value()).collect(),
        ))
    }

    type Ref = VecRef;
    type Mut = VecMut;
}

impl<S1: Schema, S2: Schema>
    SchemaOwnedConst<ManuallyDrop<[(Cow<'static, str>, SchemaNode<'static>); 2]>> for (S1, S2)
{
    const SCHEMA_OWNED_CONST: ManuallyDrop<[(Cow<'static, str>, SchemaNode<'static>); 2]> =
        ManuallyDrop::new([
            (Cow::Borrowed(""), S1::SCHEMA_NODE),
            (Cow::Borrowed(""), S2::SCHEMA_NODE),
        ]);
}

impl<A: Schema, B: Schema> Schema for (A, B) {
    const SCHEMA_NODE: SchemaNode<'static> =
        { SchemaNode::Product(VecOrSlice::Const(&Self::SCHEMA_OWNED_CONST)) };

    fn value<'a>(&'a self) -> Value<'a> {
        // TODO: avoid allocation here
        Value::Product(Cow::Owned(vec![self.0.value(), self.1.value()]))
    }

    type Ref = Tuple2Ref<A, B>;
    type Mut = Tuple2Mut<A, B>;
}

pub struct Tuple2Ref<A: Schema, B: Schema>(A::Ref, B::Ref);
pub struct Tuple2Mut<A: Schema, B: Schema>(A::Mut, B::Mut);
impl<A: Schema, B: Schema> WithPath for Tuple2Ref<A, B> {
    fn from_path(path: Cow<'_, [u32]>) -> Self {
        Self(
            A::Ref::from_path(path.iter().copied().chain([0]).collect()),
            B::Ref::from_path(path.iter().copied().chain([1]).collect()),
        )
    }

    fn path(&self) -> &[u32] {
        self.0.path().split_last().unwrap().1
    }
}
impl<A: Schema, B: Schema> WithPath for Tuple2Mut<A, B> {
    fn from_path(path: Cow<'_, [u32]>) -> Self {
        Self(
            A::Mut::from_path(path.iter().copied().chain([0]).collect()),
            B::Mut::from_path(path.iter().copied().chain([1]).collect()),
        )
    }

    fn path(&self) -> &[u32] {
        self.0.path().split_last().unwrap().1
    }
}

impl<S1: Schema, S2: Schema, S3: Schema>
    SchemaOwnedConst<ManuallyDrop<[(Cow<'static, str>, SchemaNode<'static>); 3]>> for (S1, S2, S3)
{
    const SCHEMA_OWNED_CONST: ManuallyDrop<[(Cow<'static, str>, SchemaNode<'static>); 3]> =
        ManuallyDrop::new([
            (Cow::Borrowed(""), S1::SCHEMA_NODE),
            (Cow::Borrowed(""), S2::SCHEMA_NODE),
            (Cow::Borrowed(""), S3::SCHEMA_NODE),
        ]);
}

impl<A: Schema, B: Schema, C: Schema> Schema for (A, B, C) {
    const SCHEMA_NODE: SchemaNode<'static> =
        { SchemaNode::Product(VecOrSlice::Const(&Self::SCHEMA_OWNED_CONST)) };

    fn value<'a>(&'a self) -> Value<'a> {
        // TODO: avoid allocation here
        Value::Product(Cow::Owned(vec![
            self.0.value(),
            self.1.value(),
            self.2.value(),
        ]))
    }

    type Ref = Tuple3Ref<A, B, C>;
    type Mut = Tuple3Mut<A, B, C>;
}

pub struct Tuple3Ref<A: Schema, B: Schema, C: Schema>(A::Ref, B::Ref, C::Ref);
pub struct Tuple3Mut<A: Schema, B: Schema, C: Schema>(A::Mut, B::Mut, C::Mut);
impl<A: Schema, B: Schema, C: Schema> WithPath for Tuple3Ref<A, B, C> {
    fn from_path(path: Cow<'_, [u32]>) -> Self {
        Self(
            A::Ref::from_path(path.iter().copied().chain([0]).collect()),
            B::Ref::from_path(path.iter().copied().chain([1]).collect()),
            C::Ref::from_path(path.iter().copied().chain([2]).collect()),
        )
    }

    fn path(&self) -> &[u32] {
        self.0.path().split_last().unwrap().1
    }
}
impl<A: Schema, B: Schema, C: Schema> WithPath for Tuple3Mut<A, B, C> {
    fn from_path(path: Cow<'_, [u32]>) -> Self {
        Self(
            A::Mut::from_path(path.iter().copied().chain([0]).collect()),
            B::Mut::from_path(path.iter().copied().chain([1]).collect()),
            C::Mut::from_path(path.iter().copied().chain([2]).collect()),
        )
    }

    fn path(&self) -> &[u32] {
        self.0.path().split_last().unwrap().1
    }
}
