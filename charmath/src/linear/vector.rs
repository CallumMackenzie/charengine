macro_rules! vec_operand {
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
        vec_operand!($CLASS, $NUM, Add, add, add_vec, add_num);
        vec_operand!($CLASS, $NUM, Sub, sub, sub_vec, sub_num);
        vec_operand!($CLASS, $NUM, Mul, mul, mul_vec, mul_num);
        vec_operand!($CLASS, $NUM, Div, div, div_vec, div_num);
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
        impl Vector<$NUM, $CLASS> for $CLASS {}
        impl CharMathCopy<$CLASS> for $CLASS {
            fn cm_copy(&self) -> $CLASS {
                $CLASS::new_arr(self.get_internal_array())
            }
        }
        impl VectorBase<$NUM> for $CLASS {
            fn len(&self) -> $NUM {
                $NUM::sqrt(self.dot(&self.cm_copy()))
            }
            fn get_internal_array(&self) -> &[$NUM] {
                &self.vec
            }
            fn get_mut_internal_array(&mut self) -> &mut [$NUM] {
                &mut self.vec
            }
            fn n_elems(&self) -> usize {
                $SIZE as usize
            }
        }
        impl AlgebraicAssignable<$CLASS> for $CLASS {}
        impl Algebraic<$CLASS, $CLASS> for $CLASS {}
        impl Algebraic<$NUM, $CLASS> for $CLASS {}
    };
}
macro_rules! vec_new_arr {
    ($CLASS:ident, $NUM:ident, $SIZE:expr) => {
        pub fn new_arr(arr: &[$NUM]) -> $CLASS {
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
        pub fn new_vec(vec: &dyn VectorBase<$NUM>) -> $CLASS {
            $CLASS::new_arr(vec.get_internal_array())
        }
    };
}
macro_rules! define_vec {
    ($NAME:ident, $NUM:ident, $LEN:expr) => {
        #[derive(Debug)]
        pub struct $NAME {
            vec: [$NUM; $LEN],
        }
        impl $NAME {
            vec_new_arr!($NAME, $NUM, $LEN);
        }
        vector_def!($NAME, $NUM, $LEN);
        vec_op_overload!($NAME, $NUM);
    };
}
macro_rules! vec_xy_impl {
    ($CLASS:ident, $NUM:ident) => {
        impl $CLASS {
            pub fn get_x(&self) -> $NUM {
                self.get_value(0)
            }
            pub fn get_y(&self) -> $NUM {
                self.get_value(1)
            }
            pub fn set_x(&mut self, v: $NUM) {
                self.set_value(0, v)
            }
            pub fn set_y(&mut self, v: $NUM) {
                self.set_value(1, v)
            }
        }
    };
}
macro_rules! vec_xyz_impl {
    ($CLASS:ident, $NUM:ident) => {
        vec_xy_impl!($CLASS, $NUM);
        impl $CLASS {
            pub fn get_z(&self) -> $NUM {
                self.get_value(2)
            }
            pub fn set_z(&mut self, v: $NUM) {
                self.set_value(2, v)
            }
        }
    };
}
macro_rules! vec_xyzw_impl {
    ($CLASS:ident, $NUM:ident) => {
        vec_xyz_impl!($CLASS, $NUM);
        impl $CLASS {
            pub fn get_w(&self) -> $NUM {
                self.get_value(3)
            }
            pub fn set_w(&mut self, v: $NUM) {
                self.set_value(3, v)
            }
        }
    };
}
macro_rules! define_vec2 {
    ($CLASS:ident, $NUM:ident) => {
        define_vec!($CLASS, $NUM, 2);
        vec_xy_impl!($CLASS, $NUM);
        impl $CLASS {
            pub fn new(x: $NUM, y: $NUM) -> $CLASS {
                $CLASS { vec: [x, y] }
            }
        }
    };
}
macro_rules! define_vec3 {
    ($CLASS:ident, $NUM:ident) => {
        define_vec!($CLASS, $NUM, 3);
        vec_xyz_impl!($CLASS, $NUM);
        impl $CLASS {
            pub fn new(x: $NUM, y: $NUM, z: $NUM) -> $CLASS {
                $CLASS { vec: [x, y, z] }
            }
        }
    };
}
macro_rules! define_vec4 {
    ($CLASS:ident, $NUM:ident) => {
        define_vec!($CLASS, $NUM, 4);
        vec_xyzw_impl!($CLASS, $NUM);
        impl $CLASS {
            pub fn new(x: $NUM, y: $NUM, z: $NUM, w: $NUM) -> $CLASS {
                $CLASS { vec: [x, y, z, w] }
            }
        }
    };
}

use crate::numeric::CharMathNumeric;
use crate::{Algebraic, AlgebraicAssignable, CharMathCopy};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

pub trait VectorBase<NUM: CharMathNumeric<NUM>> {
    fn get_internal_array(&self) -> &[NUM];
    fn get_mut_internal_array(&mut self) -> &mut [NUM];
    fn n_elems(&self) -> usize;
    fn len(&self) -> NUM;

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
    CharMathCopy<VEC>
    + VectorBase<NUM>
    + Index<usize, Output = NUM>
    + IndexMut<usize, Output = NUM>
    + Algebraic<VEC, VEC>
    + Algebraic<NUM, VEC>
    + AlgebraicAssignable<VEC>
{
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
    pub fn vec_dot<T: Copy + crate::numeric::CharMathNumeric<T>>(a: Vec<T>, b: Vec<T>) -> T {
        assert!(a.len() == b.len(), "Array lengths not equal");
        let mut ret = T::zero();
        for i in 0..a.len() {
            ret = ret + (a[i] * b[i]);
        }
        ret
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
