use crate::{Algebraic, AlgebraicAssignable};

pub trait CharMathNumeric<NUM>: Algebraic<NUM, NUM> + AlgebraicAssignable<NUM> + Copy {
    fn sqrt(a: NUM) -> NUM;
    fn zero() -> NUM;
}

#[macro_export]
macro_rules! charmath_numeric {
    ($NUM:ident, $CLOSEST_FLOAT:ident) => {
        impl CharMathNumeric<$NUM> for $NUM {
            fn zero() -> $NUM {
                0 as $NUM
            }
            fn sqrt(a: $NUM) -> $NUM {
                $CLOSEST_FLOAT::sqrt(a as $CLOSEST_FLOAT) as $NUM
            }
        }
        impl Algebraic<$NUM, $NUM> for $NUM {}
        impl AlgebraicAssignable<$NUM> for $NUM {}
    };
}

charmath_numeric!(f32, f32);
charmath_numeric!(f64, f64);

charmath_numeric!(i8, f32);
charmath_numeric!(i16, f32);
charmath_numeric!(i32, f32);
charmath_numeric!(i64, f64);
charmath_numeric!(i128, f64);
charmath_numeric!(isize, f64);

charmath_numeric!(u8, f32);
charmath_numeric!(u16, f32);
charmath_numeric!(u32, f32);
charmath_numeric!(u64, f64);
charmath_numeric!(u128, f64);
charmath_numeric!(usize, f64);
