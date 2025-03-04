use crate::{ChainExpression, Expression};

pub trait Chain<Rhs: Expression>: Expression + Sized {
    fn chain(self, rhs: Rhs) -> ChainExpression<Self, Rhs>;
}

impl<L: Expression, R: Expression> Chain<R> for L {
    fn chain(self, rhs: R) -> ChainExpression<Self, R> {
        ChainExpression(self, rhs)
    }
}
