#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn js_log_string(a: &str);
}

use crate::linear::vector::VectorBase;
use crate::numeric::CharMathNumeric;
use crate::CharMathCopy;
use std::ops::{Index, IndexMut};

#[cfg(target_family = "wasm")]
use crate::linear::vector::{
    Vec2f32, Vec2f64, Vec2i32, Vec2i64, Vec3f32, Vec3f64, Vec3i32, Vec3i64, Vec4f32, Vec4f64,
    Vec4i32, Vec4i64, Vector,
};

impl<N: CharMathNumeric<N>> CharMathCopy<Vec<Vec<N>>> for Vec<Vec<N>> {
    fn cm_copy(&self) -> Self {
        let mut ret = Vec::<Vec<N>>::with_capacity(self.len());
        for i in 0..self.len() {
            ret.push(Vec::<N>::with_capacity(self[i].len()));
            for j in 0..self[i].len() {
                ret[i].push(self[i][j]);
            }
        }
        ret
    }
}

pub trait MatrixBase<N: CharMathNumeric<N>> {
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
    fn get_value_ref(&self, h: usize, w: usize) -> &N;
    fn get_value_ref_mut(&mut self, h: usize, w: usize) -> &mut N;

    fn get_col_vec(&self, index: usize) -> Vec<N> {
        let mut ret = Vec::<N>::with_capacity(self.get_height());
        for i in 0..self.get_height() {
            ret.push(*self.get_value_ref(i, index));
        }
        ret
    }
    fn get_row_vec(&self, index: usize) -> Vec<N> {
        let mut ret = Vec::<N>::with_capacity(self.get_width());
        for i in 0..self.get_width() {
            ret.push(*self.get_value_ref(index, i));
        }
        ret
    }
    fn flatten(&self) -> Vec<N> {
        let wid = self.get_width();
        let mut ret = Vec::<N>::with_capacity(wid * self.get_height());
        for i in 0..self.get_height() {
            for j in 0..wid {
                ret.push(*self.get_value_ref(i, j));
            }
        }
        ret
    }
    fn is_square(&self) -> bool {
        self.get_width() == self.get_height()
    }
}

pub trait Matrix<N: CharMathNumeric<N>, MAT: Matrix<N, MAT>>:
    MatrixBase<N> + CharMathCopy<MAT>
{
    fn from_vec(vec: Vec<Vec<N>>) -> MAT;
    fn from_flat(arr: &[N], h: usize, w: usize) -> MAT;

    fn from_matrix(mat: &dyn MatrixBase<N>) -> MAT {
        Self::from_flat(&mat.flatten(), mat.get_width(), mat.get_height())
    }
    fn mul_mat(&self, rhs: &dyn MatrixBase<N>) -> MAT {
        assert!(
            self.get_width() == rhs.get_height() && self.get_height() == rhs.get_width(),
            "Matrix sizes not valid for multiplication."
        );
        let mut ret = Self::from_flat(&[], self.get_height(), rhs.get_width());
        for y in 0..ret.get_height() {
            for x in 0..ret.get_width() {
                for i in 0..self.get_width() {
                    *ret.get_value_ref_mut(y, x) +=
                        *self.get_value_ref(y, i) * *rhs.get_value_ref(i, x);
                }
            }
        }
        ret
    }
    fn mul_row_vec<T: VectorBase<N> + CharMathCopy<T>>(&self, vb: &T) -> T {
        assert!(
            self.get_height() == vb.n_elems(),
            "Vector size incompatible with matrix."
        );
        let mut ret = vb.cm_copy();
        for i in 0..self.get_width() {
            ret.set_value(
                i,
                crate::linear::vector::vector_utils::array_dot::<N>(
                    vb.get_internal_array(),
                    &self.get_col_vec(i),
                ),
            );
        }
        ret
    }
    fn mul_col_vec<T: VectorBase<N> + CharMathCopy<T>>(&self, vb: &T) -> T {
        assert!(
            self.get_width() == vb.n_elems(),
            "Vector size ({:?}) incompatible with matrix size ({}).", 
			vb.n_elems(), self.get_width()
        );
        let mut ret = vb.cm_copy();
        for i in 0..self.get_height() {
            ret.set_value(
                i,
                crate::linear::vector::vector_utils::array_dot::<N>(
                    vb.get_internal_array(),
                    &self.get_row_vec(i),
                ),
            );
        }
        ret
    }
    fn num_operand(&self, n: N, operand: fn(N, N) -> N) -> MAT {
        let mut ret = self.cm_copy();
        for i in 0..self.get_height() {
            for j in 0..self.get_width() {
                *ret.get_value_ref_mut(i, j) = operand(*self.get_value_ref(i, j), n);
            }
        }
        ret
    }
    fn mul_num(&self, n: N) -> MAT {
        self.num_operand(n, |l, r| l * r)
    }
    fn div_num(&self, n: N) -> MAT {
        self.num_operand(n, |l, r| l / r)
    }
    fn add_num(&self, n: N) -> MAT {
        self.num_operand(n, |l, r| l + r)
    }
    fn sub_num(&self, n: N) -> MAT {
        self.num_operand(n, |l, r| l - r)
    }
}
pub trait SquareMatrix<N: CharMathNumeric<N>, MAT: SquareMatrix<N, MAT>>: Matrix<N, MAT> {
    fn adjoint(&self) -> MAT {
        let mut adj = self.cm_copy();
        let mut sign: N;
        for i in 0..self.get_size() {
            for j in 0..self.get_size() {
                sign = if (i + j) % 2 == 0 {
                    N::one()
                } else {
                    N::neg(N::one())
                };
                let cofactor = self.get_cofactor(i as i32, j as i32, self.get_size() as i32);
                *adj.get_value_ref_mut(j, i) =
                    sign * cofactor.determinant_recursive((self.get_size() - 1) as u32);
            }
        }
        adj
    }
    fn inverse(&self) -> MAT {
        self.adjoint().div_num(self.determinant())
    }
    fn determinant(&self) -> N {
        self.determinant_recursive(self.get_size() as u32)
    }
    fn determinant_recursive(&self, n: u32) -> N {
        self.get_size();
        if n == 1u32 {
            *self.get_value_ref(0, 0)
        } else if n == 2u32 {
            (*self.get_value_ref(0, 0) * *self.get_value_ref(1, 1))
                - (*self.get_value_ref(0, 1) * *self.get_value_ref(1, 0))
        } else {
            let mut det = N::zero();
            let mut sign = N::one();
            for i in 0..n {
                det = det
                    + (sign
                        * *self.get_value_ref(0, i as usize)
                        * self
                            .get_cofactor(0, i as i32, n as i32)
                            .determinant_recursive(n - 1u32));
                sign = N::neg(sign);
            }
            det
        }
    }
    fn get_cofactor(&self, p: i32, q: i32, n: i32) -> MAT {
        self.get_size();
        let mut ret = self.cm_copy();
        let (mut i, mut j) = (0i32, 0i32);
        for row in 0..n {
            for col in 0..n {
                if row != p && col != q {
                    *ret.get_value_ref_mut(i as usize, j as usize) =
                        *self.get_value_ref(row as usize, col as usize);
                    j += 1;
                    if j == (n - 1i32) {
                        j = 0;
                        i += 1;
                    }
                }
            }
        }
        ret
    }
    fn get_size(&self) -> usize {
        assert!(self.is_square(), "Matrix must be square.");
        self.get_height()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct GenericMatrix<N: CharMathNumeric<N>> {
    mat: Vec<Vec<N>>,
}
impl<N: CharMathNumeric<N>> Index<usize> for GenericMatrix<N> {
    type Output = Vec<N>;
    fn index(&self, i: usize) -> &Self::Output {
        &self.mat[i]
    }
}
impl<N: CharMathNumeric<N>> IndexMut<usize> for GenericMatrix<N> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.mat[i]
    }
}
impl<N: CharMathNumeric<N>> MatrixBase<N> for GenericMatrix<N> {
    fn get_width(&self) -> usize {
        if self.mat.len() == 0 {
            0
        } else {
            self.mat[0].len()
        }
    }
    fn get_height(&self) -> usize {
        self.mat.len()
    }
    fn get_value_ref(&self, h: usize, w: usize) -> &N {
        &self.mat[h][w]
    }
    fn get_value_ref_mut(&mut self, h: usize, w: usize) -> &mut N {
        &mut self.mat[h][w]
    }
}
impl<N: CharMathNumeric<N>> GenericMatrix<N> {
    pub fn sized(h: usize, w: usize) -> Self {
        Self::from_flat(&[], h, w)
    }
}
impl<N: CharMathNumeric<N>> CharMathCopy<GenericMatrix<N>> for GenericMatrix<N> {
    fn cm_copy(&self) -> Self {
        GenericMatrix::<N> {
            mat: self.mat.cm_copy(),
        }
    }
}
impl<N: CharMathNumeric<N>> Matrix<N, Self> for GenericMatrix<N> {
    fn from_vec(vec: Vec<Vec<N>>) -> Self {
        GenericMatrix::<N> { mat: vec }
    }
    fn from_flat(arr: &[N], h: usize, w: usize) -> Self {
        let mut internal_vec = Vec::<Vec<N>>::with_capacity(h);
        for i in 0..h {
            internal_vec.push(Vec::<N>::with_capacity(w));
            for j in 0..w {
                if (i * w + j) < arr.len() {
                    internal_vec[i].push(arr[i * w + j]);
                } else {
                    internal_vec[i].push(N::zero());
                }
            }
        }
        Self::from_vec(internal_vec)
    }
}
impl<N: CharMathNumeric<N>> SquareMatrix<N, GenericMatrix<N>> for GenericMatrix<N> {}

#[derive(Debug)]
#[repr(C)]
pub struct Mat4<N: CharMathNumeric<N>> {
    mat: [[N; 4]; 4],
}
impl<N: CharMathNumeric<N>> Index<usize> for Mat4<N> {
    type Output = [N; 4];
    fn index(&self, i: usize) -> &Self::Output {
        &self.mat[i]
    }
}
impl<N: CharMathNumeric<N>> IndexMut<usize> for Mat4<N> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.mat[i]
    }
}
impl<N: CharMathNumeric<N>> MatrixBase<N> for Mat4<N> {
    fn get_width(&self) -> usize {
        4usize
    }
    fn get_height(&self) -> usize {
        4usize
    }
    fn get_value_ref(&self, h: usize, w: usize) -> &N {
        &self.mat[h][w]
    }
    fn get_value_ref_mut(&mut self, h: usize, w: usize) -> &mut N {
        &mut self.mat[h][w]
    }
}
impl<N: CharMathNumeric<N>> CharMathCopy<Mat4<N>> for Mat4<N> {
    fn cm_copy(&self) -> Self {
        Mat4::<N> { mat: self.mat }
    }
}
impl<N: CharMathNumeric<N>> Matrix<N, Self> for Mat4<N> {
    fn from_vec(vec: Vec<Vec<N>>) -> Self {
        assert!(vec.len() == 4, "Input vec height must be 4.");
        let mut internal_vec = [[N::zero(); 4]; 4];
        for i in 0..vec.len() {
            assert!(vec[i].len() == 4, "Input vec width must be 4.");
            for j in 0..vec[i].len() {
                internal_vec[i][j] = vec[i][j];
            }
        }
        Mat4::<N> { mat: internal_vec }
    }
    fn from_flat(arr: &[N], h: usize, w: usize) -> Self {
        assert!(
            h == 4 && w == 4,
            "Cannot create a Mat4 from a non-4x4 matrix."
        );
        let mut internal_vec = [[N::zero(); 4]; 4];
        for i in 0..h {
            for j in 0..w {
                if (i * w + j) < arr.len() {
                    internal_vec[i][j] = arr[i * w + j];
                } else {
                    internal_vec[i][j] = N::zero();
                }
            }
        }
        Mat4::<N> { mat: internal_vec }
    }
}
impl<N: CharMathNumeric<N>> SquareMatrix<N, Mat4<N>> for Mat4<N> {
	fn inverse(&self) -> Self {
        let mut matrix = Self::from_flat(&[], 4, 4);
        matrix.mat[0][0] = self.mat[0][0]; matrix.mat[0][1] = self.mat[1][0]; matrix.mat[0][2] = self.mat[2][0]; matrix.mat[0][3] = N::zero();
        matrix.mat[1][0] = self.mat[0][1]; matrix.mat[1][1] = self.mat[1][1]; matrix.mat[1][2] = self.mat[2][1]; matrix.mat[1][3] = N::zero();
        matrix.mat[2][0] = self.mat[0][2]; matrix.mat[2][1] = self.mat[1][2]; matrix.mat[2][2] = self.mat[2][2]; matrix.mat[2][3] = N::zero();
        matrix.mat[3][0] = N::neg(self.mat[3][0] * matrix.mat[0][0] + self.mat[3][1] * matrix.mat[1][0] + self.mat[3][2] * matrix.mat[2][0]);
        matrix.mat[3][1] = N::neg(self.mat[3][0] * matrix.mat[0][1] + self.mat[3][1] * matrix.mat[1][1] + self.mat[3][2] * matrix.mat[2][1]);
        matrix.mat[3][2] = N::neg(self.mat[3][0] * matrix.mat[0][2] + self.mat[3][1] * matrix.mat[1][2] + self.mat[3][2] * matrix.mat[2][2]);
        matrix.mat[3][3] = N::one();
        matrix
	}
}

#[derive(Debug)]
#[repr(C)]
pub struct Mat2<N: CharMathNumeric<N>> {
    mat: [[N; 2]; 2],
}
impl<N: CharMathNumeric<N>> Index<usize> for Mat2<N> {
    type Output = [N; 2];
    fn index(&self, i: usize) -> &Self::Output {
        &self.mat[i]
    }
}
impl<N: CharMathNumeric<N>> IndexMut<usize> for Mat2<N> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.mat[i]
    }
}
impl<N: CharMathNumeric<N>> MatrixBase<N> for Mat2<N> {
    fn get_width(&self) -> usize {
        2usize
    }
    fn get_height(&self) -> usize {
        2usize
    }
    fn get_value_ref(&self, h: usize, w: usize) -> &N {
        &self.mat[h][w]
    }
    fn get_value_ref_mut(&mut self, h: usize, w: usize) -> &mut N {
        &mut self.mat[h][w]
    }
}
impl<N: CharMathNumeric<N>> CharMathCopy<Mat2<N>> for Mat2<N> {
    fn cm_copy(&self) -> Self {
        Mat2::<N> { mat: self.mat }
    }
}
impl<N: CharMathNumeric<N>> Matrix<N, Self> for Mat2<N> {
    fn from_vec(vec: Vec<Vec<N>>) -> Self {
        assert!(vec.len() == 2, "Input vec height must be 2.");
        let mut internal_vec = [[N::zero(); 2]; 2];
        for i in 0..vec.len() {
            assert!(vec[i].len() == 2, "Input vec width must be 2.");
            for j in 0..vec[i].len() {
                internal_vec[i][j] = vec[i][j];
            }
        }
        Mat2::<N> { mat: internal_vec }
    }
    fn from_flat(arr: &[N], h: usize, w: usize) -> Self {
        assert!(
            h == 2 && w == 2,
            "Cannot create a Mat2 from a non-2x2 matrix."
        );
        let mut internal_vec = [[N::zero(); 2]; 2];
        for i in 0..h {
            for j in 0..w {
                if (i * w + j) < arr.len() {
                    internal_vec[i][j] = arr[i * w + j];
                } else {
                    internal_vec[i][j] = N::zero();
                }
            }
        }
        Mat2::<N> { mat: internal_vec }
    }
}
impl<N: CharMathNumeric<N>> SquareMatrix<N, Mat2<N>> for Mat2<N> {}

pub type Mat4D = Mat4<f64>;
pub type Mat4F = Mat4<f32>;
pub type Mat2D = Mat2<f64>;
pub type Mat2F = Mat2<f32>;

pub mod matrices {
    use crate::linear::matrix::{GenericMatrix, Mat2, Mat4, Matrix, MatrixBase};
    use crate::linear::quaternion::Quaternion;
    use crate::linear::vector::{Vec3, Vec4, VectorBase};
    use crate::numeric::CharMathNumeric;

    pub fn identity<N: CharMathNumeric<N>>(size: usize) -> GenericMatrix<N> {
        let mut ret = GenericMatrix::<N>::sized(size, size);
        for i in 0..size {
            *ret.get_value_ref_mut(i, i) = N::one();
        }
        ret
    }
    pub fn scale<N: CharMathNumeric<N>>(size: usize, scales: &[N]) -> GenericMatrix<N> {
        let mut ret = GenericMatrix::<N>::sized(size, size);
        for i in 0..size {
            if i < scales.len() {
                *ret.get_value_ref_mut(i, i) = scales[i];
            } else {
                *ret.get_value_ref_mut(i, i) = N::one();
            }
        }
        ret
    }
    pub fn scale_vector<N: CharMathNumeric<N>, V: VectorBase<N>>(v: &V) -> GenericMatrix<N> {
        scale::<N>(v.n_elems(), v.get_internal_array())
    }
    pub fn scale_3d<N: CharMathNumeric<N>, V: VectorBase<N>>(v: &V) -> Mat4<N> {
        Mat4::<N>::from_matrix(&scale::<N>(4, v.get_internal_array()))
    }
    pub fn scale_2d<N: CharMathNumeric<N>, V: VectorBase<N>>(v: &V) -> Mat2<N> {
        Mat2::<N>::from_matrix(&scale::<N>(2, v.get_internal_array()))
    }
    pub fn translation<N: CharMathNumeric<N>>(size: usize, trans: &[N]) -> GenericMatrix<N> {
        let mut ret = identity::<N>(size);
        for i in 0..(size - 1usize) {
            if i < trans.len() {
                *ret.get_value_ref_mut(size - 1usize, i) = trans[i];
            } else {
                break;
            }
        }
        ret
    }
    pub fn translation_vector<N: CharMathNumeric<N>, V: VectorBase<N>>(v: &V) -> GenericMatrix<N> {
        translation::<N>(v.n_elems() + 1usize, v.get_internal_array())
    }
    pub fn translation_3d<N: CharMathNumeric<N>, V: VectorBase<N>>(v: &V) -> Mat4<N> {
        Mat4::<N>::from_matrix(&translation::<N>(4, v.get_internal_array()))
    }
    pub fn rotation_2d<N: CharMathNumeric<N>>(rot: N) -> Mat2<N> {
        Mat2::<N>::from_flat(
            &[N::cos(rot), N::neg(N::sin(rot)), N::sin(rot), N::cos(rot)],
            2,
            2,
        )
    }
    pub fn rotation_euler_num<N: CharMathNumeric<N>>(x: N, y: N, z: N) -> Mat4<N> {
        let mut rot_x = identity::<N>(4);
        rot_x[1][1] = N::cos(x);
        rot_x[2][2] = N::cos(x);
        rot_x[1][2] = N::sin(x);
        rot_x[2][1] = N::neg(N::sin(x));
        let mut rot_y = identity::<N>(4);
        rot_y[0][0] = N::cos(y);
        rot_y[2][2] = N::cos(y);
        rot_y[0][2] = N::sin(y);
        rot_y[2][0] = N::neg(N::sin(y));
        let mut rot_z = identity::<N>(4);
        rot_z[0][0] = N::cos(z);
        rot_z[1][1] = N::cos(z);
        rot_z[0][1] = N::sin(z);
        rot_z[1][0] = N::neg(N::sin(z));
        Mat4::<N>::from_matrix(&rot_z.mul_mat(&rot_y).mul_mat(&rot_x))
    }
    pub fn rotation_euler<N: CharMathNumeric<N>, V: VectorBase<N>>(v: &V) -> Mat4<N> {
        rotation_euler_num::<N>(v.get_value(0), v.get_value(1), v.get_value(2))
    }
    pub fn perspective<N: CharMathNumeric<N>>(fov: N, aspect: N, near: N, far: N) -> Mat4<N> {
        let mut ret = Mat4::from_flat(&[], 4, 4);
        let fov_rad = N::one() / N::tan(N::to_radians(fov * N::half()));
        ret[0][0] = aspect * fov_rad;
        ret[1][1] = fov_rad;
        ret[2][2] = far / (far - near);
        ret[3][2] = (N::neg(far) * near) / (far - near);
        ret[2][3] = N::one();
        ret[3][3] = N::zero();
        ret
    }
    pub fn look_at_3d<N: CharMathNumeric<N>, V: Vec3<N, V>>(
        pos: &V,
        target: &V,
        up: &V,
    ) -> Mat4<N> {
        let new_forward = target.sub_vec(pos).normalized();
        let a = new_forward.mul_num(up.dot(&new_forward));
        let new_up = up.sub_vec(&a).normalized();
        let new_right = new_up.cross(&new_forward);
        let new_pos = pos;
        Mat4::<N>::from_flat(
            &[
                new_right[0],
                new_right[1],
                new_right[2],
                N::zero(),
                new_up[0],
                new_up[1],
                new_up[2],
                N::zero(),
                new_forward[0],
                new_forward[1],
                new_forward[2],
                N::zero(),
                new_pos[0],
                new_pos[1],
                new_pos[2],
                N::one(),
            ],
            4,
            4,
        )
    }
    // #[cfg_attr(target_family = "wasm", wasm_bindgen)]
    pub fn rotation_quaternion_num<N: CharMathNumeric<N>>(x: N, y: N, z: N, w: N) -> Mat4<N> {
        let two = N::two();
        let one = N::one();
        let zero = N::zero();
        Mat4::<N>::from_flat(
            &[
                one - two * y * y - two * z * z,
                two * x * y - two * z * w,
                two * x * z + two * y * w,
                zero,
                two * x * y + two * z * w,
                one - two * x * x - two * z * z,
                two * y * z - two * x * w,
                zero,
                two * x * z - two * y * w,
                two * y * z + two * x * w,
                one - two * x * x - two * y * y,
                zero,
                zero,
                zero,
                zero,
                one,
            ],
            4,
            4,
        )
    }
    pub fn rotation_quaternion<N: CharMathNumeric<N>>(q: &Quaternion<N>) -> Mat4<N> {
        rotation_quaternion_num::<N>(q.get_x(), q.get_y(), q.get_z(), q.get_w())
    }
}

macro_rules! gen_wasm_square_matrix {
    ($CLASS:ident, $NUM:ident, $SZ:expr, $VEC:ident) => {
        #[cfg_attr(target_family = "wasm", wasm_bindgen)]
        #[derive(Debug)]
        #[repr(C)]
        pub struct $CLASS {
            mat: [[$NUM; $SZ]; $SZ],
        }
        #[cfg(target_family = "wasm")]
        #[wasm_bindgen]
        impl $CLASS {
            #[wasm_bindgen(js_name = identity)]
            pub fn widentity() -> $CLASS {
                Self::from_matrix(&matrices::identity::<$NUM>($SZ))
            }
            #[wasm_bindgen(js_name = scale)]
            pub fn wscale_vector(vec: &$VEC) -> $CLASS {
                Self::from_matrix(&matrices::scale_vector::<$NUM, $VEC>(vec))
            }
            #[wasm_bindgen(constructor)]
            pub fn wfrom_flat(arr: &[$NUM]) -> $CLASS {
                Self::from_flat(arr, $SZ, $SZ)
            }
            #[wasm_bindgen(js_name = print)]
            pub fn wprint(&self) {
                js_log_string(&format!("{:?}", self));
            }
            #[wasm_bindgen(js_name = copy)]
            pub fn wcopy(&self) -> $CLASS {
                self.cm_copy()
            }
            #[wasm_bindgen(js_name = getWidth)]
            pub fn wget_width(&self) -> f64 {
                self.get_width() as f64
            }
            #[wasm_bindgen(js_name = getHeight)]
            pub fn wget_height(&self) -> f64 {
                self.get_height() as f64
            }
            #[wasm_bindgen(js_name = getSize)]
            pub fn wget_size(&self) -> f64 {
                self.get_size() as f64
            }
            #[wasm_bindgen(js_name = f64At)]
            pub fn wget_elem_f64(&self, y: f64, x: f64) -> f64 {
                *self.get_value_ref(y as usize, x as usize) as f64
            }
            #[wasm_bindgen(js_name = f32At)]
            pub fn wget_elem_f32(&self, y: f64, x: f64) -> f32 {
                *self.get_value_ref(y as usize, x as usize) as f32
            }
            #[wasm_bindgen(js_name = i64At)]
            pub fn wget_elem_i64(&self, y: f64, x: f64) -> i64 {
                *self.get_value_ref(y as usize, x as usize) as i64
            }
            #[wasm_bindgen(js_name = i32At)]
            pub fn wget_elem_i32(&self, y: f64, x: f64) -> i32 {
                *self.get_value_ref(y as usize, x as usize) as i32
            }
            #[wasm_bindgen(js_name = isSquare)]
            pub fn wis_square(&self) -> bool {
                self.is_square()
            }
            #[wasm_bindgen(js_name = getColVec)]
            pub fn wget_col_vec(&self, index: f64) -> $VEC {
                $VEC::new_arr(&self.get_col_vec(index as usize))
            }
            #[wasm_bindgen(js_name = getRowVec)]
            pub fn wget_row_vec(&self, index: f64) -> $VEC {
                $VEC::new_arr(&self.get_row_vec(index as usize))
            }
            #[wasm_bindgen(js_name = mulMat)]
            pub fn wmul_mat(&self, o: &$CLASS) -> $CLASS {
                self.mul_mat(o)
            }
            #[wasm_bindgen(js_name = mulRowVec)]
            pub fn wmul_row_vec(&self, o: &$VEC) -> $VEC {
                self.mul_row_vec(o)
            }
            #[wasm_bindgen(js_name = mulColVec)]
            pub fn wmul_col_vec(&self, o: &$VEC) -> $VEC {
                self.mul_col_vec(o)
            }
            #[wasm_bindgen(js_name = mulNum)]
            pub fn wmul_num(&self, o: f64) -> $CLASS {
                self.mul_num(o as $NUM)
            }
            #[wasm_bindgen(js_name = divNum)]
            pub fn wdiv_num(&self, o: f64) -> $CLASS {
                self.div_num(o as $NUM)
            }
            #[wasm_bindgen(js_name = addNum)]
            pub fn wadd_num(&self, o: f64) -> $CLASS {
                self.add_num(o as $NUM)
            }
            #[wasm_bindgen(js_name = subNum)]
            pub fn wsub_num(&self, o: f64) -> $CLASS {
                self.sub_num(o as $NUM)
            }
            #[wasm_bindgen(js_name = adjoint)]
            pub fn wadjoint(&self) -> $CLASS {
                self.adjoint()
            }
            #[wasm_bindgen(js_name = inverse)]
            pub fn winverse(&self) -> $CLASS {
                self.inverse()
            }
            #[wasm_bindgen(js_name = determinant)]
            pub fn wdeterminant(&self) -> f64 {
                self.determinant() as f64
            }
            #[wasm_bindgen(js_name = toString)]
            pub fn wto_string(&self) -> String {
                format!("{:?}", self).into()
            }
        }
        impl CharMathCopy<$CLASS> for $CLASS {
            fn cm_copy(&self) -> $CLASS {
                $CLASS { mat: self.mat }
            }
        }
        impl Index<usize> for $CLASS {
            type Output = [$NUM; $SZ];
            fn index(&self, i: usize) -> &Self::Output {
                &self.mat[i]
            }
        }
        impl IndexMut<usize> for $CLASS {
            fn index_mut(&mut self, i: usize) -> &mut Self::Output {
                &mut self.mat[i]
            }
        }
        impl MatrixBase<$NUM> for $CLASS {
            fn get_width(&self) -> usize {
                $SZ as usize
            }
            fn get_height(&self) -> usize {
                $SZ as usize
            }
            fn get_value_ref(&self, h: usize, w: usize) -> &$NUM {
                &self.mat[h][w]
            }
            fn get_value_ref_mut(&mut self, h: usize, w: usize) -> &mut $NUM {
                &mut self.mat[h][w]
            }
        }
        impl Matrix<$NUM, Self> for $CLASS {
            fn from_vec(vec: Vec<Vec<$NUM>>) -> Self {
                assert!(vec.len() == $SZ, "Input vec height incompatible.");
                let mut internal_vec = [[$NUM::zero(); $SZ]; $SZ];
                for i in 0..vec.len() {
                    assert!(vec[i].len() == 2, "Input vec width incompatible.");
                    for j in 0..vec[i].len() {
                        internal_vec[i][j] = vec[i][j];
                    }
                }
                $CLASS { mat: internal_vec }
            }
            fn from_flat(arr: &[$NUM], h: usize, w: usize) -> Self {
                assert!(h == $SZ && w == $SZ, "Matrix size incompatible.");
                let mut internal_vec = [[$NUM::zero(); $SZ]; $SZ];
                for i in 0..h {
                    for j in 0..w {
                        if (i * w + j) < arr.len() {
                            internal_vec[i][j] = arr[i * w + j];
                        } else {
                            internal_vec[i][j] = $NUM::zero();
                        }
                    }
                }
                $CLASS { mat: internal_vec }
            }
        }
        impl SquareMatrix<$NUM, $CLASS> for $CLASS {}
    };
}
macro_rules! gen_wasm_sq_mat4 {
    ($CLASS:ident, $NUM:ident, $VEC:ident, $SVEC:ident, $QUA:ident) => {
        gen_wasm_square_matrix!($CLASS, $NUM, 4, $VEC);
        #[cfg(target_family = "wasm")]
        #[wasm_bindgen]
        impl $CLASS {
            #[wasm_bindgen(js_name = translation)]
            pub fn wtranslation(vec: &$SVEC) -> $CLASS {
                $CLASS::from_matrix(&matrices::translation_3d::<$NUM, $SVEC>(vec))
            }
            #[wasm_bindgen(js_name = rotationEuler)]
            pub fn wrotation_euler(vec: &$SVEC) -> $CLASS {
                $CLASS::from_matrix(&matrices::rotation_euler::<$NUM, $SVEC>(vec))
            }
            #[wasm_bindgen(js_name = rotationQuaternion)]
            pub fn wrotation_quaternion(q: &$QUA) -> $CLASS {
                $CLASS::from_matrix(&matrices::rotation_quaternion_num::<$NUM>(
                    q.get_x(),
                    q.get_y(),
                    q.get_z(),
                    q.get_w(),
                ))
            }
            #[wasm_bindgen(js_name = lookAt)]
            pub fn wlook_at(pos: &$SVEC, target: &$SVEC, up: &$SVEC) -> $CLASS {
                $CLASS::from_matrix(&matrices::look_at_3d::<$NUM, $SVEC>(pos, target, up))
            }
            #[wasm_bindgen(js_name = perspective)]
            pub fn wperspective(fov: f64, aspect: f64, near: f64, far: f64) -> $CLASS {
                $CLASS::from_matrix(&matrices::perspective::<$NUM>(
                    fov as $NUM,
                    aspect as $NUM,
                    near as $NUM,
                    far as $NUM,
                ))
            }
        }
    };
}
macro_rules! gen_wasm_sq_mat2 {
    ($CLASS:ident, $NUM:ident, $VEC:ident) => {
        gen_wasm_square_matrix!($CLASS, $NUM, 2, $VEC);
        #[cfg(target_family = "wasm")]
        #[wasm_bindgen]
        impl $CLASS {
            #[wasm_bindgen(js_name = rotation)]
            pub fn wrotation_2d(num: f64) -> $CLASS {
                $CLASS::from_matrix(&matrices::rotation_2d::<$NUM>(num as $NUM))
            }
        }
    };
}

#[cfg(target_family = "wasm")]
use crate::linear::quaternion::{Quaternionf32, Quaternionf64, Quaternioni32, Quaternioni64};
#[cfg(target_family = "wasm")]
use crate::linear::vector::Vec4;

gen_wasm_sq_mat4!(Mat4f64, f64, Vec4f64, Vec3f64, Quaternionf64);
gen_wasm_sq_mat4!(Mat4f32, f32, Vec4f32, Vec3f32, Quaternionf32);
gen_wasm_sq_mat4!(Mat4i64, i64, Vec4i64, Vec3i64, Quaternioni64);
gen_wasm_sq_mat4!(Mat4i32, i32, Vec4i32, Vec3i32, Quaternioni32);

gen_wasm_square_matrix!(Mat3f64, f64, 3, Vec3f64);
gen_wasm_square_matrix!(Mat3f32, f32, 3, Vec3f32);
gen_wasm_square_matrix!(Mat3i64, i64, 3, Vec3i64);
gen_wasm_square_matrix!(Mat3i32, i32, 3, Vec3i32);

gen_wasm_sq_mat2!(Mat2f64, f64, Vec2f64);
gen_wasm_sq_mat2!(Mat2f32, f32, Vec2f32);
gen_wasm_sq_mat2!(Mat2i64, i64, Vec2i64);
gen_wasm_sq_mat2!(Mat2i32, i32, Vec2i32);
