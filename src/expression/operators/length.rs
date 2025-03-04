use crate::{Expression, LengthExpression};

pub trait Length: Expression + Sized {
    fn length(self) -> LengthExpression<Self>;
}

impl<Lhs: Expression<Target = Vec<T>>, T: Expression> Length for Lhs {
    fn length(self) -> LengthExpression<Self> {
        LengthExpression(self)
    }
}
