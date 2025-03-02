use std::{marker::PhantomData, num::NonZeroU32};

use crate::{
    BoolOperators, Expression, GetExpression, Key, NonZeroUint32Equal, OptionOperators,
    PathExpression, Schema, SlotMap,
};

use super::{FlattenOperator, Set};

pub trait SlotMapOperators<K: Key, Ke: Expression<Target = K>, T: Schema + Send + Sync> {
    fn get(self, key: Ke) -> impl Expression<Target = Option<T>>;

    fn remove(self, key: Ke) -> impl Expression<Target = Option<T>>;

    // fn insert(self, value: T) -> impl Expression<Target = K>;
}

impl<
        K: Key,
        Ke: Expression<Target = K> + Clone,
        T: Schema + Expression<Target = T> + Send + Sync,
        E: Expression<Target = SlotMap<K, T>>,
    > SlotMapOperators<K, Ke, T> for E
{
    fn get(self, key: Ke) -> impl Expression<Target = Option<T>> {
        let key_index = GetExpression::<Ke, u32, u32>(key.clone(), 0, PhantomData);
        let key_generation = GetExpression::<Ke, u32, NonZeroU32>(key, 1, PhantomData);

        let get_result = GetExpression::<
            Self,
            GetExpression<Ke, u32, u32>,
            Option<(NonZeroU32, Option<T>)>,
        >(self, key_index, PhantomData);

        get_result
            .map(|slot| {
                slot.0
                    .equal(key_generation)
                    .if_else::<Option<T>, PathExpression<Option<T>>, Option<T>>(slot.1, None)
            })
            .flatten()
    }

    fn remove(self, key: Ke) -> impl Expression<Target = Option<T>> {
        let key_index = GetExpression::<Ke, u32, u32>(key.clone(), 0, PhantomData);
        let key_generation = GetExpression::<Ke, u32, NonZeroU32>(key, 1, PhantomData);

        let get_result = GetExpression::<
            Self,
            GetExpression<Ke, u32, u32>,
            Option<(NonZeroU32, Option<T>)>,
        >(self, key_index, PhantomData);

        get_result
            .map(|slot| {
                slot.0
                    .clone()
                    .equal(key_generation)
                    .if_else::<Option<T>, _, Option<T>>(slot.1.set(Option::<T>::None), None)
            })
            .flatten()
    }

    // fn insert(self, value: T) -> impl Expression<Target = K> {
    //     todo!()
    // }
}
