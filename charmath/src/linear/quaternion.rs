macro_rules! charmath_quat_operand {
    ($OPNAME:ident, $OPFNNAME:ident, $VECOPNAME:ident, $NUMOPNAME:ident) => {
        impl<N: CharMathNumeric<N>> $OPNAME<&Quaternion<N>> for &Quaternion<N> {
            type Output = Quaternion<N>;
            fn $OPFNNAME(self, rhs: &Quaternion<N>) -> Quaternion<N> {
                self.$VECOPNAME(rhs)
            }
        }
        impl<N: CharMathNumeric<N>> $OPNAME<N> for &Quaternion<N> {
            type Output = Quaternion<N>;
            fn $OPFNNAME(self, rhs: N) -> Quaternion<N> {
                self.$NUMOPNAME(rhs)
            }
        }
        impl<N: CharMathNumeric<N>> $OPNAME<Quaternion<N>> for Quaternion<N> {
            type Output = Quaternion<N>;
            fn $OPFNNAME(self, rhs: Quaternion<N>) -> Quaternion<N> {
                self.$VECOPNAME(&rhs)
            }
        }
        impl<N: CharMathNumeric<N>> $OPNAME<N> for Quaternion<N> {
            type Output = Quaternion<N>;
            fn $OPFNNAME(self, rhs: N) -> Quaternion<N> {
                self.$NUMOPNAME(rhs)
            }
        }
        impl<N: CharMathNumeric<N>> $OPNAME<Quaternion<N>> for &Quaternion<N> {
            type Output = Quaternion<N>;
            fn $OPFNNAME(self, rhs: Quaternion<N>) -> Quaternion<N> {
                self.$VECOPNAME(&rhs)
            }
        }
        impl<N: CharMathNumeric<N>> $OPNAME<&Quaternion<N>> for Quaternion<N> {
            type Output = Quaternion<N>;
            fn $OPFNNAME(self, rhs: &Quaternion<N>) -> Quaternion<N> {
                self.$VECOPNAME(rhs)
            }
        }
    };
}
macro_rules! quat_assign_operand {
    ($ASOP:ident, $ASOPNAME:ident, $VCALL:ident, $NVCALL:ident) => {
        impl<N: CharMathNumeric<N>> $ASOP<Quaternion<N>> for Quaternion<N> {
            fn $ASOPNAME(&mut self, rhs: Quaternion<N>) {
                self.$VCALL(&rhs);
            }
        }
        impl<N: CharMathNumeric<N>> $ASOP<&Quaternion<N>> for Quaternion<N> {
            fn $ASOPNAME(&mut self, rhs: &Quaternion<N>) {
                self.$VCALL(rhs);
            }
        }
        impl<N: CharMathNumeric<N>> $ASOP<Quaternion<N>> for &mut Quaternion<N> {
            fn $ASOPNAME(&mut self, rhs: Quaternion<N>) {
                self.$VCALL(&rhs);
            }
        }
        impl<N: CharMathNumeric<N>> $ASOP<&Quaternion<N>> for &mut Quaternion<N> {
            fn $ASOPNAME(&mut self, rhs: &Quaternion<N>) {
                self.$VCALL(rhs);
            }
        }
        impl<N: CharMathNumeric<N>> $ASOP<N> for Quaternion<N> {
            fn $ASOPNAME(&mut self, rhs: N) {
                self.$NVCALL(rhs);
            }
        }
        impl<N: CharMathNumeric<N>> $ASOP<N> for &mut Quaternion<N> {
            fn $ASOPNAME(&mut self, rhs: N) {
                self.$NVCALL(rhs);
            }
        }
    };
}

use crate::linear::vector::{Vec3, Vec4, Vector, VectorBase};
use crate::numeric::CharMathNumeric;
use crate::CharMathCopy;
use crate::{Algebraic, AlgebraicAssignable};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug)]
pub struct Quaternion<N: CharMathNumeric<N>> {
    vec: [N; 4],
}

charmath_quat_operand!(Add, add, add_vec, add_num);
charmath_quat_operand!(Sub, sub, sub_vec, sub_num);
charmath_quat_operand!(Mul, mul, mul_vec, mul_num);
charmath_quat_operand!(Div, div, div_vec, div_num);

quat_assign_operand!(AddAssign, add_assign, add_eq_vec, add_eq_num);
quat_assign_operand!(SubAssign, sub_assign, sub_eq_vec, sub_eq_num);
quat_assign_operand!(MulAssign, mul_assign, mul_eq_vec, mul_eq_num);
quat_assign_operand!(DivAssign, div_assign, div_eq_vec, div_eq_num);

impl<N: CharMathNumeric<N>> Algebraic<Quaternion<N>, Quaternion<N>> for Quaternion<N> {}
impl<N: CharMathNumeric<N>> Algebraic<N, Quaternion<N>> for Quaternion<N> {}
impl<N: CharMathNumeric<N>> AlgebraicAssignable<Quaternion<N>> for Quaternion<N> {}

impl<N: CharMathNumeric<N>> CharMathCopy<Quaternion<N>> for Quaternion<N> {
    fn cm_copy(&self) -> Quaternion<N> {
        Quaternion::<N> { vec: self.vec }
    }
}
impl<N: CharMathNumeric<N>> VectorBase<N> for Quaternion<N> {
    fn get_internal_array(&self) -> &[N] {
        &self.vec
    }
    fn get_mut_internal_array(&mut self) -> &mut [N] {
        &mut self.vec
    }
}
impl<N: CharMathNumeric<N>> Index<usize> for Quaternion<N> {
    type Output = N;
    fn index(&self, i: usize) -> &N {
        &self.vec[i]
    }
}
impl<N: CharMathNumeric<N>> IndexMut<usize> for Quaternion<N> {
    fn index_mut(&mut self, i: usize) -> &mut N {
        &mut self.vec[i]
    }
}
impl<N: CharMathNumeric<N>> Vector<N, Quaternion<N>> for Quaternion<N> {
    fn new_arr(arr: &[N]) -> Quaternion<N> {
        let mut ret = Quaternion::<N> {
            vec: [N::zero(); 4],
        };
        for i in 0..4 {
            if i < arr.len() {
                ret.vec[i] = arr[i];
            }
        }
        ret
    }
}
impl<N: CharMathNumeric<N>> Vec4<N, Quaternion<N>> for Quaternion<N> {
    fn new(x: N, y: N, z: N, w: N) -> Quaternion<N> {
        Quaternion::<N> { vec: [x, y, z, w] }
    }
}
impl<N: CharMathNumeric<N>> Quaternion<N> {
    pub fn complex_real<V: Vec3<N, V>>(complex: &V, real: N) -> Quaternion<N> {
        Self::new(complex.get_x(), complex.get_y(), complex.get_z(), real)
    }
    pub fn angle_axis<V: Vec3<N, V>>(angle: N, axis: &V) -> Quaternion<N> {
        Self::complex_real::<V>(
            &axis.mul_num(N::sin(angle * N::half())),
            N::cos(angle * N::half()),
        )
    }
    pub fn get_complex<V: Vec4<N, V>>(&self) -> V {
        V::new_arr(&[self.get_x(), self.get_y(), self.get_z()])
    }
    pub fn set_complex<V: Vec4<N, V>>(&mut self, complex: &V) {
        self.set_x(complex.get_x());
        self.set_y(complex.get_y());
        self.set_z(complex.get_z());
    }
}
