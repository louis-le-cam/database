use std::{
    future::Future,
    io,
    iter::{Enumerate, FilterMap},
    marker::PhantomData,
    num::NonZeroU32,
};

use tokio::io::AsyncWriteExt;

use crate::{expression_discriminant, io_error, Expression, Schema, SlotMapExpression};

pub trait Key {
    fn new(index: u32, generation: NonZeroU32) -> Self;

    fn index(&self) -> u32;

    fn generation(&self) -> NonZeroU32;
}

#[macro_export]
macro_rules! make_keys {
    ($($(#[$meta:meta])* $vis:vis struct $name:ident;)+) => {
        $(
            $(#[$meta])*
            $vis struct $name {
                index: u32,
                generation: ::core::num::NonZeroU32,
            }

            impl $crate::Key for $name {
                fn new(index: u32, generation: ::core::num::NonZeroU32) -> Self {
                    Self { index, generation }
                }

                fn index(&self) -> u32 {
                    self.index
                }

                fn generation(&self) -> ::core::num::NonZeroU32 {
                    self.generation
                }
            }
        )+
    };
}

make_keys! {
    pub struct DefaultKey;
}

#[derive(Debug)]
pub struct SlotMap<K: Key, T>(Vec<(NonZeroU32, Option<T>)>, PhantomData<K>);

impl<K: Key, T> SlotMap<K, T> {
    pub fn new() -> Self {
        Self(Vec::new(), PhantomData)
    }
}

impl<K: Key, T> FromIterator<T> for SlotMap<K, T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|value| (NonZeroU32::new(1).unwrap(), Some(value)))
                .collect(),
            PhantomData,
        )
    }
}

impl<K: Key, T> IntoIterator for SlotMap<K, T> {
    type Item = (K, T);

    type IntoIter = FilterMap<
        Enumerate<std::vec::IntoIter<(NonZeroU32, Option<T>)>>,
        fn((usize, (NonZeroU32, Option<T>))) -> Option<(K, T)>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0
            .into_iter()
            .enumerate()
            .filter_map(|(index, (generation, value))| {
                value.map(|value| (K::new(index as u32, generation), value))
            })
    }
}

impl<K: Key + Send + Sync, T: Schema + Send + Sync> Schema for SlotMap<K, T> {
    type Expression = SlotMapExpression<K, T>;

    fn write_schema(
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        Vec::<(u32, Option<T>)>::write_schema(write)
    }

    fn write_value(
        &self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> + Send {
        async move {
            write
                .write_u32(self.0.len().try_into().map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "list value length doesn't fit into a 32 bit unsigned integer",
                    )
                })?)
                .await?;

            for (generation, value) in &self.0 {
                generation.get().write_value(write).await?;
                value.write_value(write).await?;
            }

            Ok(())
        }
    }

    fn read_value(
        read: &mut (impl tokio::io::AsyncReadExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<Self>> + Send {
        async {
            let length: usize = read.read_u32().await?.try_into().map_err(|_| {
                io_error!(
                    OutOfMemory,
                    "list value length doesn't fit into a pointer sized unsigned integer",
                )
            })?;

            let mut values = Vec::new();
            values.try_reserve(length).map_err(|_| {
                io_error!(OutOfMemory, "allocation of memory for list values failed")
            })?;

            for _ in 0..length {
                let Some(generation) = NonZeroU32::new(u32::read_value(read).await?) else {
                    return Err(io_error!(InvalidData, "generation in slotmap is zero"));
                };

                values.push((generation, Option::<T>::read_value(read).await?));
            }

            Ok(SlotMap(values, PhantomData))
        }
    }
}

// TODO: find a way to pass hashmap containing expressions in query
impl<K: Key + Send + Sync, T: Schema + Send + Sync> Expression for SlotMap<K, T> {
    type Target = SlotMap<K, T>;

    fn write(
        self,
        write: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> impl Future<Output = io::Result<()>> {
        async move {
            write.write_u8(expression_discriminant::LIST).await?;

            write
                .write_u32(self.0.len().try_into().map_err(|_| {
                    io_error!(
                        OutOfMemory,
                        "list expression length doesn't fit into a 32 bit unsigned integer",
                    )
                })?)
                .await?;

            for (generation, value) in self.0 {
                write.write_u8(expression_discriminant::VALUE).await?;

                <(u32, Option<T>)>::write_schema(write).await?;

                generation.get().write_value(write).await?;
                value.write_value(write).await?;
            }

            Ok(())
        }
    }
}
