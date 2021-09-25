use crate::{Algebraic, AlgebraicAssignable};

pub trait CharMathNumeric<NUM>: Algebraic<NUM, NUM> + AlgebraicAssignable<NUM> + Copy {
    fn sqrt(a: NUM) -> NUM;
    fn zero() -> NUM;
    fn one() -> NUM;
    fn neg(a: NUM) -> NUM;
}

#[macro_export]
macro_rules! charmath_numeric_signed {
    ($NUM:ident, $CLOSEST_FLOAT:ident) => {
        impl CharMathNumeric<$NUM> for $NUM {
            fn zero() -> $NUM {
                0 as $NUM
            }
            fn sqrt(a: $NUM) -> $NUM {
                $CLOSEST_FLOAT::sqrt(a as $CLOSEST_FLOAT) as $NUM
            }
            fn neg(a: $NUM) -> $NUM {
                -a
            }
            fn one() -> $NUM {
                1 as $NUM
            }
        }
        impl Algebraic<$NUM, $NUM> for $NUM {}
        impl AlgebraicAssignable<$NUM> for $NUM {}
    };
}
#[macro_export]
macro_rules! charmath_numeric_unsigned {
    ($NUM:ident, $CLOSEST_FLOAT:ident) => {
        impl CharMathNumeric<$NUM> for $NUM {
            fn zero() -> $NUM {
                0 as $NUM
            }
            fn sqrt(a: $NUM) -> $NUM {
                $CLOSEST_FLOAT::sqrt(a as $CLOSEST_FLOAT) as $NUM
            }
            fn neg(_: $NUM) -> $NUM {
                panic!("Unsigned numbers cannot be negative.")
            }
            fn one() -> $NUM {
                1 as $NUM
            }
        }
        impl Algebraic<$NUM, $NUM> for $NUM {}
        impl AlgebraicAssignable<$NUM> for $NUM {}
    };
}

charmath_numeric_signed!(f32, f32);
charmath_numeric_signed!(f64, f64);

charmath_numeric_signed!(i8, f32);
charmath_numeric_signed!(i16, f32);
charmath_numeric_signed!(i32, f32);
charmath_numeric_signed!(i64, f64);
charmath_numeric_signed!(i128, f64);
charmath_numeric_signed!(isize, f64);

charmath_numeric_unsigned!(u8, f32);
charmath_numeric_unsigned!(u16, f32);
charmath_numeric_unsigned!(u32, f32);
charmath_numeric_unsigned!(u64, f64);
charmath_numeric_unsigned!(u128, f64);
charmath_numeric_unsigned!(usize, f64);
