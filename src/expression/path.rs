use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    io,
    marker::PhantomData,
    time::Duration,
};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, io_error, Expression, Key, OptionMapped, Schema, SlotMap};

pub trait FromPath {
    fn from_path(path: Vec<u32>) -> Self;
}

pub struct UnitExpression(Vec<u32>);
pub struct BoolExpression(Vec<u32>);
pub struct StringExpression(Vec<u32>);

pub struct Uint8Expression(Vec<u32>);
pub struct Uint16Expression(Vec<u32>);
pub struct Uint32Expression(Vec<u32>);
pub struct Uint64Expression(Vec<u32>);
pub struct Uint128Expression(Vec<u32>);
pub struct Int8Expression(Vec<u32>);
pub struct Int16Expression(Vec<u32>);
pub struct Int32Expression(Vec<u32>);
pub struct Int64Expression(Vec<u32>);
pub struct Int128Expression(Vec<u32>);
pub struct Float32Expression(Vec<u32>);
pub struct Float64Expression(Vec<u32>);

pub struct OptionExpression<S: Schema + Send + Sync>(Vec<u32>, PhantomData<S>);
pub struct OptionMappedExpression<Some, None>(Vec<u32>, PhantomData<(Some, None)>);
pub struct DurationExpression(Vec<u32>);
pub struct VecExpression<T: Schema + Send + Sync>(Vec<u32>, PhantomData<T>);
pub struct HashMapExpression<K: Schema + Send + Sync + Eq + Hash, V: Schema + Send + Sync>(
    Vec<u32>,
    PhantomData<(K, V)>,
);
pub struct HashSetExpression<T: Schema + Send + Sync + Eq + Hash>(Vec<u32>, PhantomData<T>);
pub struct SlotMapExpression<K: Key + Send + Sync, T: Schema + Send + Sync>(
    Vec<u32>,
    PhantomData<(K, T)>,
);

macro_rules! impl_path_expr {
    ($([$($gen_decl:tt)*] $type:ty, $target:ty, ($($extra_fields:tt)*);)+) => {
        $(
            impl<$($gen_decl)*> Clone for $type {
                fn clone(&self) -> Self {
                    Self(self.0.clone(), $($extra_fields)*)
                }
            }

            impl<$($gen_decl)*> $crate::FromPath for $type {
                fn from_path(path: ::std::vec::Vec<u32>) -> Self {
                    Self(path, $($extra_fields)*)
                }
            }

            impl<$($gen_decl)*> $crate::Expression for $type {
                type Target = $target;

                async fn write(
                    self,
                    write: &mut (impl ::tokio::io::AsyncWriteExt + ::std::marker::Unpin),
                ) -> ::std::io::Result<()> {
                    write
                        .write_u8($crate::expression_discriminant::PATH)
                        .await?;
                    write
                        .write_u32(self.0.len().try_into().map_err(|_| {
                            $crate::io_error!(
                                OutOfMemory,
                                "path expression length doesn't fit into a 32 bit unsigned integer",
                            )
                        })?)
                        .await?;

                    for segment in &self.0 {
                        write.write_u32(*segment).await?;
                    }

                    Ok(())
                }
            }
        )+
    };
}

impl_path_expr!(
    [] UnitExpression, (), ();
    [] BoolExpression, bool, ();
    [] StringExpression, String, ();

    [] Uint8Expression, u8,();
    [] Uint16Expression, u16,();
    [] Uint32Expression, u32,();
    [] Uint64Expression, u64,();
    [] Uint128Expression, u128,();
    [] Int8Expression, i8,();
    [] Int16Expression, i16,();
    [] Int32Expression, i32,();
    [] Int64Expression, i64,();
    [] Int128Expression, i128,();
    [] Float32Expression, f32,();
    [] Float64Expression, f64,();

    [S: Schema + Send + Sync] OptionExpression<S>, Option<S>, (PhantomData);
    [Some: Schema + Send + Sync, None: Schema + Send + Sync] OptionMappedExpression<Some, None>, OptionMapped<Some, None>, (PhantomData);
    [T: Schema + Send + Sync] VecExpression<T>, Vec<T>, (::core::marker::PhantomData);
    [] DurationExpression, Duration, ();
    [K: Schema + Send + Sync + Eq + Hash, V: Schema + Send + Sync] HashMapExpression<K, V>, HashMap<K, V>, (PhantomData);
    [T: Schema + Send + Sync + Eq + Hash] HashSetExpression<T>, HashSet<T>, (PhantomData);
    [K: Key + Send + Sync, T: Schema + Send + Sync] SlotMapExpression<K, T>, SlotMap<K, T>, (PhantomData);
);

macro_rules! make_tuple_path_expr {
    ($($last_index:tt $name:ident $($field:ident)*;)*) => {
        $(
            pub struct $name<$($field: Schema + Send + Sync,)*>($(pub $field::Expression,)* Vec<u32>);

            impl<$($field: Schema + Send + Sync,)*> Clone for $name<$($field,)*> {
                fn clone(&self) -> Self {
                    Self::from_path(self.$last_index.clone())
                }
            }

            impl<$($field: Schema + Send + Sync,)*> Expression for $name<$($field,)*> {
                type Target = ($($field,)*);

                async fn write(self, write: &mut (impl AsyncWriteExt + Unpin)) -> io::Result<()> {
                    write.write_u8(expression_discriminant::PATH).await?;
                    write
                        .write_u32(self.$last_index.len().try_into().map_err(|_| {
                            io_error!(
                                OutOfMemory,
                                "path expression length doesn't fit into a 32 bit unsigned integer",
                            )
                        })?)
                        .await?;

                    for segment in &self.$last_index {
                        write.write_u32(*segment).await?;
                    }

                    Ok(())
                }
            }

            impl<$($field: Schema + Send + Sync,)*> FromPath for $name<$($field,)*> {
                fn from_path(path: Vec<u32>) -> Self {
                    let mut i = 0;
                    #[allow(unused_assignments)]
                    Self($($field::Expression::from_path((path.iter().copied().chain([i]).collect(), i += 1).0)),*, path)
                }
            }
        )*
    };
}

make_tuple_path_expr!(
    1 TupleExpression1 A;
    2 TupleExpression2 A B;
    3 TupleExpression3 A B C;
    4 TupleExpression4 A B C D;
    5 TupleExpression5 A B C D E;
    6 TupleExpression6 A B C D E F;
    7 TupleExpression7 A B C D E F G;
    8 TupleExpression8 A B C D E F G H;
    9 TupleExpression9 A B C D E F G H I;
    10 TupleExpression10 A B C D E F G H I J;
    11 TupleExpression11 A B C D E F G H I J K;
    12 TupleExpression12 A B C D E F G H I J K L;
    13 TupleExpression13 A B C D E F G H I J K L M;
    14 TupleExpression14 A B C D E F G H I J K L M N;
    15 TupleExpression15 A B C D E F G H I J K L M N O;
    16 TupleExpression16 A B C D E F G H I J K L M N O P;
);
