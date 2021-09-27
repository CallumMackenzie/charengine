pub mod linear;
pub mod numeric;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

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

#[cfg(not(target_family = "wasm"))]
pub fn link_test() {
    println!("CharMath link verified");
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
pub fn link_test() {
    alert(&"CharMath link verified");
}
