#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn js_log_string(a: &str);
}

use crate::numeric::CharMathNumeric;
use crate::{Algebraic, AlgebraicAssignable, CharMathCopy};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

macro_rules! charmath_def_operand {
    ($CLASS:ident, $NUM:ident, $OPNAME:ident, $CALL1:ident, $CALL2:ident, $CALL3:ident) => {
        impl $OPNAME<&$CLASS> for &$CLASS {
            type Output = $CLASS;
            fn $CALL1(self, rhs: &$CLASS) -> $CLASS {
                self.$CALL2(rhs)
            }
        }
        impl $OPNAME<$NUM> for &$CLASS {
            type Output = $CLASS;
            fn $CALL1(self, rhs: $NUM) -> $CLASS {
                self.$CALL3(rhs)
            }
        }
        impl $OPNAME<$CLASS> for $CLASS {
            type Output = $CLASS;
            fn $CALL1(self, rhs: $CLASS) -> $CLASS {
                self.$CALL2(&rhs)
            }
        }
        impl $OPNAME<$NUM> for $CLASS {
            type Output = $CLASS;
            fn $CALL1(self, rhs: $NUM) -> $CLASS {
                self.$CALL3(rhs)
            }
        }
        impl $OPNAME<$CLASS> for &$CLASS {
            type Output = $CLASS;
            fn $CALL1(self, rhs: $CLASS) -> $CLASS {
                self.$CALL2(&rhs)
            }
        }
        impl $OPNAME<&$CLASS> for $CLASS {
            type Output = $CLASS;
            fn $CALL1(self, rhs: &$CLASS) -> $CLASS {
                self.$CALL2(rhs)
            }
        }
    };
}
macro_rules! vec_assign_operand {
    ($CLASS:ident, $NUM:ident, $ASOP:ident, $ASOPNAME:ident, $VCALL:ident, $NVCALL:ident) => {
        impl $ASOP<$CLASS> for $CLASS {
            fn $ASOPNAME(&mut self, rhs: $CLASS) {
                self.$VCALL(&rhs);
            }
        }
        impl $ASOP<&$CLASS> for $CLASS {
            fn $ASOPNAME(&mut self, rhs: &$CLASS) {
                self.$VCALL(rhs);
            }
        }
        impl $ASOP<$CLASS> for &mut $CLASS {
            fn $ASOPNAME(&mut self, rhs: $CLASS) {
                self.$VCALL(&rhs);
            }
        }
        impl $ASOP<&$CLASS> for &mut $CLASS {
            fn $ASOPNAME(&mut self, rhs: &$CLASS) {
                self.$VCALL(rhs);
            }
        }
        impl $ASOP<$NUM> for $CLASS {
            fn $ASOPNAME(&mut self, rhs: $NUM) {
                self.$NVCALL(rhs);
            }
        }
        impl $ASOP<$NUM> for &mut $CLASS {
            fn $ASOPNAME(&mut self, rhs: $NUM) {
                self.$NVCALL(rhs);
            }
        }
    };
}

macro_rules! vec_op_overload {
    ($CLASS:ident, $NUM:ident) => {
        charmath_def_operand!($CLASS, $NUM, Add, add, add_vec, add_num);
        charmath_def_operand!($CLASS, $NUM, Sub, sub, sub_vec, sub_num);
        charmath_def_operand!($CLASS, $NUM, Mul, mul, mul_vec, mul_num);
        charmath_def_operand!($CLASS, $NUM, Div, div, div_vec, div_num);
        vec_assign_operand!($CLASS, $NUM, AddAssign, add_assign, add_eq_vec, add_eq_num);
        vec_assign_operand!($CLASS, $NUM, SubAssign, sub_assign, sub_eq_vec, sub_eq_num);
        vec_assign_operand!($CLASS, $NUM, MulAssign, mul_assign, mul_eq_vec, mul_eq_num);
        vec_assign_operand!($CLASS, $NUM, DivAssign, div_assign, div_eq_vec, div_eq_num);
        impl Index<usize> for $CLASS {
            type Output = $NUM;
            fn index(&self, index: usize) -> &$NUM {
                self.get_value_ref(index)
            }
        }
        impl IndexMut<usize> for $CLASS {
            fn index_mut(&mut self, index: usize) -> &mut $NUM {
                self.get_value_ref_mut(index)
            }
        }
    };
}

macro_rules! vector_def {
    ($CLASS:ident, $NUM:ident, $SIZE:expr) => {
        impl Vector<$NUM, $CLASS> for $CLASS {
            fn new_arr(arr: &[$NUM]) -> $CLASS {
                let mut vec_arr = [$NUM::zero(); $SIZE];
                for i in 0..$SIZE {
                    if i < arr.len() {
                        vec_arr[i] = arr[i];
                    } else {
                        vec_arr[i] = $NUM::zero();
                    }
                }
                $CLASS { vec: vec_arr }
            }
        }
        impl CharMathCopy<$CLASS> for $CLASS {
            fn cm_copy(&self) -> $CLASS {
                $CLASS::new_arr(self.get_internal_array())
            }
        }
        impl VectorBase<$NUM> for $CLASS {
            fn get_internal_array(&self) -> &[$NUM] {
                &self.vec
            }
            fn get_mut_internal_array(&mut self) -> &mut [$NUM] {
                &mut self.vec
            }
        }
        impl AlgebraicAssignable<$CLASS> for $CLASS {}
        impl Algebraic<$CLASS, $CLASS> for $CLASS {}
        impl Algebraic<$NUM, $CLASS> for $CLASS {}
    };
}
macro_rules! define_vec {
    ($CLASS:ident, $NUM:ident, $LEN:expr) => {
        #[cfg_attr(target_family = "wasm", wasm_bindgen)]
        #[derive(Debug, Clone, Copy)]
        #[repr(C)]
        pub struct $CLASS {
            vec: [$NUM; $LEN],
        }
        #[cfg(target_family = "wasm")]
        #[wasm_bindgen]
        impl $CLASS {
            #[wasm_bindgen(js_name = i64At)]
            pub fn wi64_val_at(&self, i: i64) -> i64 {
                self.vec[i as usize] as i64
            }
            #[wasm_bindgen(js_name = f64At)]
            pub fn wf64_val_at(&self, i: i64) -> f64 {
                self.vec[i as usize] as f64
            }
            #[wasm_bindgen(js_name = i32At)]
            pub fn wi32_val_at(&self, i: i64) -> i32 {
                self.vec[i as usize] as i32
            }
            #[wasm_bindgen(js_name = f32At)]
            pub fn wf32_val_at(&self, i: i64) -> f32 {
                self.vec[i as usize] as f32
            }
            #[wasm_bindgen(js_name = toString)]
            pub fn wto_string(&self) -> String {
                format!("{:?}", self).into()
            }
            #[wasm_bindgen(js_name = print)]
            pub fn wprint(&self) {
                js_log_string(&format!("{:?}", self));
            }
            #[wasm_bindgen(js_name = len)]
            pub fn wlen(&self) -> f64 {
                self.len() as f64
            }
            #[wasm_bindgen(js_name = nElems)]
            pub fn wn_elems(&self) -> f64 {
                self.n_elems() as f64
            }
            #[wasm_bindgen(js_name = copy)]
            pub fn wcopy(&self) -> $CLASS {
                self.cm_copy()
            }
            #[wasm_bindgen(js_name = dot)]
            pub fn wdot(&self, o: &$CLASS) -> f64 {
                self.dot(o) as f64
            }
            #[wasm_bindgen(js_name = mulVec)]
            pub fn wmul_vec(&self, o: &$CLASS) -> $CLASS {
                self.mul_vec(o)
            }
            #[wasm_bindgen(js_name = addVec)]
            pub fn wadd_vec(&self, o: &$CLASS) -> $CLASS {
                self.add_vec(o)
            }
            #[wasm_bindgen(js_name = divVec)]
            pub fn wdiv_vec(&self, o: &$CLASS) -> $CLASS {
                self.div_vec(o)
            }
            #[wasm_bindgen(js_name = subVec)]
            pub fn wsub_vec(&self, o: &$CLASS) -> $CLASS {
                self.sub_vec(o)
            }
            #[wasm_bindgen(js_name = set)]
            pub fn wset(&mut self, o: &$CLASS) {
                self.set(o);
            }
            #[wasm_bindgen(js_name = mulNum)]
            pub fn wmul_num(&self, o: f64) -> $CLASS {
                self.mul_num(o as $NUM)
            }
            #[wasm_bindgen(js_name = addNum)]
            pub fn wadd_num(&self, o: f64) -> $CLASS {
                self.add_num(o as $NUM)
            }
            #[wasm_bindgen(js_name = divNum)]
            pub fn wdiv_num(&self, o: f64) -> $CLASS {
                self.div_num(o as $NUM)
            }
            #[wasm_bindgen(js_name = subNum)]
            pub fn wsub_num(&self, o: f64) -> $CLASS {
                self.sub_num(o as $NUM)
            }
            #[wasm_bindgen(js_name = normalized)]
            pub fn wnormalized(&self) -> $CLASS {
                self.normalized()
            }
            #[wasm_bindgen(js_name = normalize)]
            pub fn wnormalize(&mut self) {
                self.normalize();
            }
        }
        vector_def!($CLASS, $NUM, $LEN);
        vec_op_overload!($CLASS, $NUM);
    };
}
macro_rules! define_vec2 {
    ($CLASS:ident, $NUM:ident) => {
        define_vec!($CLASS, $NUM, 2);
        impl Vec2<$NUM, $CLASS> for $CLASS {
            fn new(x: $NUM, y: $NUM) -> $CLASS {
                $CLASS { vec: [x, y] }
            }
        }
        #[cfg(target_family = "wasm")]
        #[wasm_bindgen]
        impl $CLASS {
            #[wasm_bindgen(constructor)]
            pub fn wnew(x: f64, y: f64) -> $CLASS {
                Self::new(x as $NUM, y as $NUM)
            }
            #[wasm_bindgen(js_name = getX)]
            pub fn wget_x(&self) -> f64 {
                self.get_x() as f64
            }
            #[wasm_bindgen(js_name = getY)]
            pub fn wget_y(&self) -> f64 {
                self.get_y() as f64
            }
            #[wasm_bindgen(js_name = setX)]
            pub fn wset_x(&mut self, n: f64) {
                self.set_x(n as $NUM);
            }
            #[wasm_bindgen(js_name = setY)]
            pub fn wset_y(&mut self, n: f64) {
                self.set_y(n as $NUM);
            }
        }
    };
}
macro_rules! define_vec3 {
    ($CLASS:ident, $NUM:ident) => {
        define_vec!($CLASS, $NUM, 3);
        impl Vec3<$NUM, $CLASS> for $CLASS {
            fn new(x: $NUM, y: $NUM, z: $NUM) -> $CLASS {
                $CLASS { vec: [x, y, z] }
            }
        }
        #[cfg(target_family = "wasm")]
        #[wasm_bindgen]
        impl $CLASS {
            #[wasm_bindgen(constructor)]
            pub fn wnew(x: f64, y: f64, z: f64) -> $CLASS {
                Self::new(x as $NUM, y as $NUM, z as $NUM)
            }
            #[wasm_bindgen(js_name = getX)]
            pub fn wget_x(&self) -> f64 {
                self.get_x() as f64
            }
            #[wasm_bindgen(js_name = getY)]
            pub fn wget_y(&self) -> f64 {
                self.get_y() as f64
            }
            #[wasm_bindgen(js_name = getZ)]
            pub fn wget_z(&self) -> f64 {
                self.get_z() as f64
            }
            #[wasm_bindgen(js_name = setX)]
            pub fn wset_x(&mut self, n: f64) {
                self.set_x(n as $NUM);
            }
            #[wasm_bindgen(js_name = setY)]
            pub fn wset_y(&mut self, n: f64) {
                self.set_y(n as $NUM);
            }
            #[wasm_bindgen(js_name = setZ)]
            pub fn wset_z(&mut self, n: f64) {
                self.set_z(n as $NUM)
            }
            #[wasm_bindgen(js_name = cross)]
            pub fn wcross(&self, o: &$CLASS) -> $CLASS {
                self.cross(o)
            }
        }
    };
}
macro_rules! define_vec4 {
    ($CLASS:ident, $NUM:ident) => {
        define_vec!($CLASS, $NUM, 4);
        impl Vec4<$NUM, $CLASS> for $CLASS {
            fn new(x: $NUM, y: $NUM, z: $NUM, w: $NUM) -> $CLASS {
                $CLASS { vec: [x, y, z, w] }
            }
        }
        #[cfg(target_family = "wasm")]
        #[wasm_bindgen]
        impl $CLASS {
            #[wasm_bindgen(constructor)]
            pub fn wnew(x: f64, y: f64, z: f64, w: f64) -> $CLASS {
                Self::new(x as $NUM, y as $NUM, z as $NUM, w as $NUM)
            }
            #[wasm_bindgen(js_name = getX)]
            pub fn wget_x(&self) -> f64 {
                self.get_x() as f64
            }
            #[wasm_bindgen(js_name = getY)]
            pub fn wget_y(&self) -> f64 {
                self.get_y() as f64
            }
            #[wasm_bindgen(js_name = getZ)]
            pub fn wget_z(&self) -> f64 {
                self.get_z() as f64
            }
            #[wasm_bindgen(js_name = getW)]
            pub fn wget_w(&self) -> f64 {
                self.get_w() as f64
            }
            #[wasm_bindgen(js_name = setX)]
            pub fn wset_x(&mut self, n: f64) {
                self.set_x(n as $NUM);
            }
            #[wasm_bindgen(js_name = setY)]
            pub fn wset_y(&mut self, n: f64) {
                self.set_y(n as $NUM);
            }
            #[wasm_bindgen(js_name = setZ)]
            pub fn wset_z(&mut self, n: f64) {
                self.set_z(n as $NUM)
            }
            #[wasm_bindgen(js_name = setW)]
            pub fn wset_w(&mut self, n: f64) {
                self.set_w(n as $NUM);
            }
        }
    };
}

pub trait VectorBase<NUM: CharMathNumeric<NUM>> {
    fn get_internal_array(&self) -> &[NUM];
    fn get_mut_internal_array(&mut self) -> &mut [NUM];

    fn n_elems(&self) -> usize {
        self.get_internal_array().len()
    }
    fn len(&self) -> NUM {
        NUM::sqrt(vector_utils::array_dot(
            self.get_internal_array(),
            self.get_internal_array(),
        ))
    }
    fn get_value(&self, index: usize) -> NUM {
        self.get_internal_array()[index]
    }
    fn get_value_ref(&self, index: usize) -> &NUM {
        &self.get_internal_array()[index]
    }
    fn get_value_ref_mut(&mut self, index: usize) -> &mut NUM {
        &mut self.get_mut_internal_array()[index]
    }
    fn set_value(&mut self, index: usize, val: NUM) {
        self.get_mut_internal_array()[index] = val;
    }
}

pub trait Vector<NUM: Copy + Algebraic<NUM, NUM> + CharMathNumeric<NUM>, VEC: Vector<NUM, VEC>>:
    CharMathCopy<VEC> + VectorBase<NUM> + Index<usize, Output = NUM> + IndexMut<usize, Output = NUM>
// + Algebraic<VEC, VEC>
// + Algebraic<NUM, VEC>
// + AlgebraicAssignable<VEC>
{
    fn new_arr(arr: &[NUM]) -> VEC;

    fn new_vec(vec: &dyn VectorBase<NUM>) -> VEC {
        Self::new_arr(vec.get_internal_array())
    }
    fn new_std_vec(vec: &Vec<NUM>) -> VEC {
        Self::new_arr(&vec[0..vec.len()])
    }
    fn dot(&self, other: &VEC) -> NUM {
        vector_utils::array_dot::<NUM>(self.get_internal_array(), other.get_internal_array())
    }
    fn normalized(&self) -> VEC {
        self.div_num(self.len())
    }
    fn normalize(&mut self) -> &Self {
        self.set(&self.normalized())
    }
    fn set(&mut self, other: &VEC) -> &mut Self {
        for i in 0..self.n_elems() {
            self.set_value(i, other.get_value(i));
        }
        self
    }
    fn num_op(&self, other: NUM, operand: fn(NUM, NUM) -> NUM) -> VEC {
        let mut ret: VEC = self.cm_copy();
        vector_utils::single_arr_op(
            self.get_internal_array(),
            ret.get_mut_internal_array(),
            other,
            operand,
        );
        ret
    }
    fn vec_op(&self, other: &VEC, operand: fn(NUM, NUM) -> NUM) -> VEC {
        let mut ret: VEC = self.cm_copy();
        vector_utils::parallel_arr_op(
            self.get_internal_array(),
            other.get_internal_array(),
            ret.get_mut_internal_array(),
            operand,
        );
        ret
    }
    fn add_vec(&self, other: &VEC) -> VEC {
        self.vec_op(other, |ls: NUM, rs: NUM| ls + rs)
    }
    fn sub_vec(&self, other: &VEC) -> VEC {
        self.vec_op(other, |ls: NUM, rs: NUM| ls - rs)
    }
    fn div_vec(&self, other: &VEC) -> VEC {
        self.vec_op(other, |ls: NUM, rs: NUM| ls / rs)
    }
    fn mul_vec(&self, other: &VEC) -> VEC {
        self.vec_op(other, |ls: NUM, rs: NUM| ls * rs)
    }
    fn add_num(&self, other: NUM) -> VEC {
        self.num_op(other, |ls: NUM, rs: NUM| ls + rs)
    }
    fn sub_num(&self, other: NUM) -> VEC {
        self.num_op(other, |ls: NUM, rs: NUM| ls - rs)
    }
    fn div_num(&self, other: NUM) -> VEC {
        self.num_op(other, |ls: NUM, rs: NUM| ls / rs)
    }
    fn mul_num(&self, other: NUM) -> VEC {
        self.num_op(other, |ls: NUM, rs: NUM| ls * rs)
    }
    fn add_eq_vec(&mut self, other: &VEC) -> &Self {
        self.set(&self.add_vec(other))
    }
    fn sub_eq_vec(&mut self, other: &VEC) -> &Self {
        self.set(&self.sub_vec(other))
    }
    fn div_eq_vec(&mut self, other: &VEC) -> &Self {
        self.set(&self.div_vec(other))
    }
    fn mul_eq_vec(&mut self, other: &VEC) -> &Self {
        self.set(&self.mul_vec(other))
    }
    fn add_eq_num(&mut self, other: NUM) -> &Self {
        self.set(&self.add_num(other))
    }
    fn sub_eq_num(&mut self, other: NUM) -> &Self {
        self.set(&self.sub_num(other))
    }
    fn mul_eq_num(&mut self, other: NUM) -> &Self {
        self.set(&self.mul_num(other))
    }
    fn div_eq_num(&mut self, other: NUM) -> &Self {
        self.set(&self.div_num(other))
    }
}

pub mod vector_utils {
    pub fn parallel_arr_op<T: Copy>(a: &[T], b: &[T], c: &mut [T], operand: fn(T, T) -> T) {
        assert!(
            (a.len() == b.len()) && (a.len() == c.len()),
            "Array lengths not equal."
        );
        for i in 0..a.len() {
            c[i] = operand(a[i], b[i]);
        }
    }
    pub fn single_arr_op<T: Copy>(a: &[T], b: &mut [T], val: T, operand: fn(T, T) -> T) {
        assert!(a.len() == b.len(), "Array lengths not equal");
        for i in 0..a.len() {
            b[i] = operand(a[i], val);
        }
    }
    pub fn array_dot<T: Copy + crate::numeric::CharMathNumeric<T>>(a: &[T], b: &[T]) -> T {
        assert!(a.len() == b.len(), "Array lengths not equal");
        let mut ret = T::zero();
        for i in 0..a.len() {
            ret = ret + (a[i] * b[i]);
        }
        ret
    }
    pub fn array_vec_dot<T: Copy + crate::numeric::CharMathNumeric<T>>(a: &[T], b: Vec<T>) -> T {
        assert!(a.len() == b.len(), "Array lengths not equal");
        let mut ret = T::zero();
        for i in 0..a.len() {
            ret = ret + (a[i] * b[i]);
        }
        ret
    }
    pub fn vec_dot<T: crate::numeric::CharMathNumeric<T>>(a: Vec<T>, b: Vec<T>) -> T {
        assert!(a.len() == b.len(), "Array lengths not equal");
        let mut ret = T::zero();
        for i in 0..a.len() {
            ret = ret + (a[i] * b[i]);
        }
        ret
    }
    pub fn array_cross<T: crate::numeric::CharMathNumeric<T>>(a: &[T], b: &[T]) -> Vec<T> {
        assert!(
            a.len() == 3 && b.len() == 3,
            "Input array lengths must be 3.",
        );
        let mut ret = Vec::<T>::with_capacity(3);
        ret.push(a[1] * b[2] - a[2] * b[1]);
        ret.push(a[2] * b[0] - a[0] * b[2]);
        ret.push(a[0] * b[1] - a[1] * b[0]);
        ret
    }
}

pub trait Vec2<N: CharMathNumeric<N>, V: Vec2<N, V>>: Vector<N, V> {
    fn new(x: N, y: N) -> V;
    fn get_x(&self) -> N {
        self.get_value(0)
    }
    fn set_x(&mut self, x: N) {
        self.set_value(0, x);
    }
    fn get_y(&self) -> N {
        self.get_value(1)
    }
    fn set_y(&mut self, y: N) {
        self.set_value(1, y);
    }
}
pub trait Vec3<N: CharMathNumeric<N>, V: Vec3<N, V>>: Vector<N, V> {
    fn new(x: N, y: N, z: N) -> V;
    fn cross(&self, other: &V) -> V {
        Self::new_std_vec(&vector_utils::array_cross::<N>(
            self.get_internal_array(),
            other.get_internal_array(),
        ))
    }
    fn get_x(&self) -> N {
        self.get_value(0)
    }
    fn set_x(&mut self, x: N) {
        self.set_value(0, x);
    }
    fn get_y(&self) -> N {
        self.get_value(1)
    }
    fn set_y(&mut self, y: N) {
        self.set_value(1, y);
    }
    fn get_z(&self) -> N {
        self.get_value(2)
    }
    fn set_z(&mut self, z: N) {
        self.set_value(2, z);
    }
}
pub trait Vec4<N: CharMathNumeric<N>, V: Vec4<N, V>>: Vector<N, V> {
    fn new(x: N, y: N, z: N, w: N) -> V;
    fn get_x(&self) -> N {
        self.get_value(0)
    }
    fn set_x(&mut self, x: N) {
        self.set_value(0, x);
    }
    fn get_y(&self) -> N {
        self.get_value(1)
    }
    fn set_y(&mut self, y: N) {
        self.set_value(1, y);
    }
    fn get_z(&self) -> N {
        self.get_value(2)
    }
    fn set_z(&mut self, z: N) {
        self.set_value(2, z);
    }
    fn get_w(&self) -> N {
        self.get_value(3)
    }
    fn set_w(&mut self, w: N) {
        self.set_value(3, w);
    }
}

define_vec2!(Vec2f32, f32);
define_vec2!(Vec2f64, f64);
define_vec2!(Vec2i16, i16);
define_vec2!(Vec2i32, i32);
define_vec2!(Vec2i64, i64);
define_vec2!(Vec2i128, i128);
define_vec2!(Vec2u8, u8);
define_vec2!(Vec2u16, u16);
define_vec2!(Vec2u32, u32);
define_vec2!(Vec2u64, u64);
define_vec2!(Vec2u128, u128);
define_vec2!(Vec2usize, usize);

define_vec3!(Vec3f32, f32);
define_vec3!(Vec3f64, f64);
define_vec3!(Vec3i16, i16);
define_vec3!(Vec3i32, i32);
define_vec3!(Vec3i64, i64);
define_vec3!(Vec3i128, i128);
define_vec3!(Vec3u8, u8);
define_vec3!(Vec3u16, u16);
define_vec3!(Vec3u32, u32);
define_vec3!(Vec3u64, u64);
define_vec3!(Vec3u128, u128);
define_vec3!(Vec3usize, usize);

define_vec4!(Vec4f32, f32);
define_vec4!(Vec4f64, f64);
define_vec4!(Vec4i16, i16);
define_vec4!(Vec4i32, i32);
define_vec4!(Vec4i64, i64);
define_vec4!(Vec4i128, i128);
define_vec4!(Vec4u8, u8);
define_vec4!(Vec4u16, u16);
define_vec4!(Vec4u32, u32);
define_vec4!(Vec4u64, u64);
define_vec4!(Vec4u128, u128);
define_vec4!(Vec4usize, usize);

pub type Vec2D = Vec2f64;
pub type Vec2F = Vec2f32;
pub type Vec3D = Vec3f64;
pub type Vec3F = Vec3f32;
pub type Vec4D = Vec4f64;
pub type Vec4F = Vec4f32;
