use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};

use crate::{EqualExpression, Expression};

macro_rules! impl_equal {
    ($($trait:ident $target:ty;)*) => {
        $(
            pub trait $trait<Rhs: Expression>: Expression + Sized {
                fn equal(self, rhs: Rhs) -> EqualExpression<Self, Rhs>;
            }

            impl<L: Expression<Target = $target>, R: Expression<Target = $target>> $trait<R> for L {
                fn equal(self, rhs: R) -> EqualExpression<Self, R> {
                    EqualExpression(self, rhs)
                }
            }
        )*
    };
}

impl_equal!(
    StringEqual String;
    Uint8Equal u8;
    Uint16Equal u16;
    Uint32Equal u32;
    Uint64Equal u64;
    Uint128Equal u128;
    Int8Equal i8;
    Int16Equal i16;
    Int32Equal i32;
    Int64Equal i64;
    Int128Equal i128;
    NonZeroUint8Equal NonZeroU8;
    NonZeroUint16Equal NonZeroU16;
    NonZeroUint32Equal NonZeroU32;
    NonZeroUint64Equal NonZeroU64;
    NonZeroUint128Equal NonZeroU128;
    NonZeroInt8Equal NonZeroI8;
    NonZeroInt16Equal NonZeroI16;
    NonZeroInt32Equal NonZeroI32;
    NonZeroInt64Equal NonZeroI64;
    NonZeroInt128Equal NonZeroI128;
);
