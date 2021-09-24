pub mod linear;
pub mod numeric;

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
pub trait Algebraic<IN, OUT>:
    Add<IN, Output = OUT> + Sub<IN, Output = OUT> + Div<IN, Output = OUT> + Mul<IN, Output = OUT>
{
}

pub trait AlgebraicAssignable<IN>:
    AddAssign<IN> + SubAssign<IN> + MulAssign<IN> + DivAssign<IN>
{
}

pub trait CharMathCopy<T> {
    fn cm_copy(&self) -> T;
}
