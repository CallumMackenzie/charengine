use crate::{Algebraic, AlgebraicAssignable};

pub trait CharMathNumeric<NUM>: Algebraic<NUM, NUM> + AlgebraicAssignable<NUM> + Copy {
    fn sqrt(a: NUM) -> NUM;
    fn zero() -> NUM;
    fn half() -> NUM;
    fn one() -> NUM;
    fn two() -> NUM;
    fn to_radians(deg: NUM) -> NUM;
    fn to_degrees(rad: NUM) -> NUM;
    fn neg(a: NUM) -> NUM;
    fn cos(a: NUM) -> NUM;
    fn sin(a: NUM) -> NUM;
    fn tan(a: NUM) -> NUM;
}

#[macro_export]
macro_rules! charmath_numeric {
    ($NUM:ident, $CLOSEST_FLOAT:ident, $NEG_SYM:expr) => {
        impl CharMathNumeric<$NUM> for $NUM {
            fn zero() -> $NUM {
                0 as $NUM
            }
            fn two() -> $NUM {
                2 as $NUM
            }
            fn sqrt(a: $NUM) -> $NUM {
                $CLOSEST_FLOAT::sqrt(a as $CLOSEST_FLOAT) as $NUM
            }
            fn one() -> $NUM {
                1 as $NUM
            }
            fn cos(a: $NUM) -> $NUM {
                $CLOSEST_FLOAT::cos(a as $CLOSEST_FLOAT) as $NUM
            }
            fn sin(a: $NUM) -> $NUM {
                $CLOSEST_FLOAT::sin(a as $CLOSEST_FLOAT) as $NUM
            }
            fn neg(a: $NUM) -> $NUM {
                $NEG_SYM as $NUM * a
            }
            fn tan(a: $NUM) -> $NUM {
                Self::sin(a) / Self::cos(a)
            }
            fn half() -> $NUM {
                0.5 as $NUM
            }
            fn to_radians(deg: $NUM) -> $NUM {
                (deg as $CLOSEST_FLOAT * (std::$CLOSEST_FLOAT::consts::PI / 180 as $CLOSEST_FLOAT))
                    as $NUM
            }
            fn to_degrees(rad: $NUM) -> $NUM {
                (rad as $CLOSEST_FLOAT * (180 as $CLOSEST_FLOAT / std::$CLOSEST_FLOAT::consts::PI))
                    as $NUM
            }
        }
        impl Algebraic<$NUM, $NUM> for $NUM {}
        impl AlgebraicAssignable<$NUM> for $NUM {}
    };
}

charmath_numeric!(f32, f32, -1);
charmath_numeric!(f64, f64, -1);

charmath_numeric!(i8, f32, -1);
charmath_numeric!(i16, f32, -1);
charmath_numeric!(i32, f32, -1);
charmath_numeric!(i64, f64, -1);
charmath_numeric!(i128, f64, -1);
charmath_numeric!(isize, f64, -1);

charmath_numeric!(u8, f32, 1);
charmath_numeric!(u16, f32, 1);
charmath_numeric!(u32, f32, 1);
charmath_numeric!(u64, f64, 1);
charmath_numeric!(u128, f64, 1);
charmath_numeric!(usize, f64, 1);
