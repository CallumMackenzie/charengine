use crate::linear::vector::VectorBase;
use crate::numeric::CharMathNumeric;
use crate::CharMathCopy;

impl<NUM: CharMathNumeric<NUM>> CharMathCopy<Vec<Vec<NUM>>> for Vec<Vec<NUM>> {
    fn cm_copy(&self) -> Self {
        let mut ret = Vec::<Vec<NUM>>::with_capacity(self.len());
        for i in 0..self.len() {
            ret.push(Vec::<NUM>::with_capacity(self[i].len()));
            for j in 0..self[i].len() {
                ret[i].push(self[i][j]);
            }
        }
        ret
    }
}

pub trait MatrixBase<NUM: CharMathNumeric<NUM>> {
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
    fn get_value_ref(&self, h: usize, w: usize) -> &NUM;
    fn get_value_ref_mut(&mut self, h: usize, w: usize) -> &mut NUM;
    fn get_col_vec(&self, index: usize) -> Vec<NUM>;
    fn get_row_vec(&self, index: usize) -> Vec<NUM>;

    fn flatten(&self) -> Vec<NUM> {
        let wid = self.get_width();
        let mut ret = Vec::<NUM>::with_capacity(wid * self.get_height());
        for i in 0..self.get_height() {
            for j in 0..wid {
                ret[i * wid + j] = *self.get_value_ref(i, j);
            }
        }
        ret
    }
    fn is_square(&self) -> bool {
        self.get_width() == self.get_height()
    }
}

pub trait Matrix<NUM: CharMathNumeric<NUM>, MAT: Matrix<NUM, MAT>>:
    MatrixBase<NUM> + CharMathCopy<MAT>
{
    fn from_vec(vec: Vec<Vec<NUM>>) -> MAT;
    fn from_flat(arr: &[NUM], h: usize, w: usize) -> MAT;
    fn sized(h: usize, w: usize) -> MAT;

    fn mul_mat(&self, rhs: &dyn MatrixBase<NUM>) -> MAT {
        assert!(
            self.get_width() == rhs.get_height() && self.get_height() == rhs.get_width(),
            "Matrix sizes not valid for multiplication."
        );
        let mut ret = Self::sized(self.get_height(), rhs.get_width());
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
    fn mul_row_vec<T: VectorBase<NUM> + CharMathCopy<T>>(&self, vb: &T) -> T {
        assert!(
            self.get_height() == vb.n_elems(),
            "Vector size incompatible with matrix."
        );
        let mut ret = vb.cm_copy();
        for i in 0..self.get_width() {
            ret.set_value(
                i,
                crate::linear::vector::vector_utils::array_dot::<NUM>(
                    vb.get_internal_array(),
                    &self.get_col_vec(i),
                ),
            );
        }
        ret
    }
    fn mul_col_vec<T: VectorBase<NUM> + CharMathCopy<T>>(&self, vb: &T) -> T {
        assert!(
            self.get_width() == vb.n_elems(),
            "Vector size incompatible with matrix."
        );
        let mut ret = vb.cm_copy();
        for i in 0..self.get_height() {
            ret.set_value(
                i,
                crate::linear::vector::vector_utils::array_dot::<NUM>(
                    vb.get_internal_array(),
                    &self.get_row_vec(i),
                ),
            );
        }
        ret
    }
    fn num_operand(&self, n: NUM, operand: fn(NUM, NUM) -> NUM) -> MAT {
        let mut ret = self.cm_copy();
        for i in 0..self.get_height() {
            for j in 0..self.get_width() {
                *ret.get_value_ref_mut(i, j) = operand(*self.get_value_ref(i, j), n);
            }
        }
        ret
    }
    fn mul_num(&self, n: NUM) -> MAT {
        self.num_operand(n, |l, r| l * r)
    }
    fn div_num(&self, n: NUM) -> MAT {
        self.num_operand(n, |l, r| l / r)
    }
    fn add_num(&self, n: NUM) -> MAT {
        self.num_operand(n, |l, r| l + r)
    }
    fn sub_num(&self, n: NUM) -> MAT {
        self.num_operand(n, |l, r| l - r)
    }
}
pub trait SquareMatrix<NUM: CharMathNumeric<NUM>, MAT: SquareMatrix<NUM, MAT>>:
    Matrix<NUM, MAT>
{
    fn adjoint(&self) -> MAT {
        let mut adj = self.cm_copy();
        let mut sign: NUM;
        for i in 0..self.get_size() {
            for j in 0..self.get_size() {
                sign = if (i + j) % 2 == 0 {
                    NUM::one()
                } else {
                    NUM::neg(NUM::one())
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
    fn determinant(&self) -> NUM {
        self.determinant_recursive(self.get_size() as u32)
    }
    fn determinant_recursive(&self, n: u32) -> NUM {
        self.get_size();
        if n == 1u32 {
            *self.get_value_ref(0, 0)
        } else if n == 2u32 {
            (*self.get_value_ref(0, 0) * *self.get_value_ref(1, 1))
                - (*self.get_value_ref(0, 1) * *self.get_value_ref(1, 0))
        } else {
            let mut det = NUM::zero();
            let mut sign = NUM::one();
            for i in 0..n {
                det = det
                    + (sign
                        * *self.get_value_ref(0, i as usize)
                        * self
                            .get_cofactor(0, i as i32, n as i32)
                            .determinant_recursive(n - 1u32));
                sign = NUM::neg(sign);
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
pub struct GenericMatrix<NUM: CharMathNumeric<NUM>> {
    mat: Vec<Vec<NUM>>,
}
impl<NUM: CharMathNumeric<NUM>> Matrix<NUM, Self> for GenericMatrix<NUM> {
    fn from_vec(vec: Vec<Vec<NUM>>) -> Self {
        GenericMatrix::<NUM> { mat: vec }
    }
    fn from_flat(arr: &[NUM], h: usize, w: usize) -> Self {
        assert!(
            arr.len() == (h * w),
            "Array size does not match widths and heights given."
        );
        let mut internal_vec = Vec::<Vec<NUM>>::with_capacity(h);
        for i in 0..h {
            internal_vec.push(Vec::<NUM>::with_capacity(w));
            for j in 0..w {
                internal_vec[i].push(arr[i * w + j]);
            }
        }
        Self::from_vec(internal_vec)
    }
    fn sized(h: usize, w: usize) -> Self {
        let mut internal_vec = Vec::<Vec<NUM>>::with_capacity(h);
        for i in 0..h {
            internal_vec.push(Vec::<NUM>::with_capacity(w));
            for _ in 0..w {
                internal_vec[i].push(NUM::zero());
            }
        }
        Self::from_vec(internal_vec)
    }
}
impl<NUM: CharMathNumeric<NUM>> CharMathCopy<GenericMatrix<NUM>> for GenericMatrix<NUM> {
    fn cm_copy(&self) -> Self {
        GenericMatrix::<NUM> {
            mat: self.mat.cm_copy(),
        }
    }
}
impl<NUM: CharMathNumeric<NUM>> MatrixBase<NUM> for GenericMatrix<NUM> {
    fn get_width(&self) -> usize {
        if self.mat.len() == 0 {
            return 0;
        }
        self.mat[0].len()
    }
    fn get_height(&self) -> usize {
        self.mat.len()
    }
    fn get_value_ref(&self, h: usize, w: usize) -> &NUM {
        &self.mat[h][w]
    }
    fn get_value_ref_mut(&mut self, h: usize, w: usize) -> &mut NUM {
        &mut self.mat[h][w]
    }
    fn get_col_vec(&self, index: usize) -> Vec<NUM> {
        let mut ret = Vec::<NUM>::with_capacity(self.get_height());
        for i in 0..self.get_height() {
            ret.push(self.mat[i][index]);
        }
        ret
    }
    fn get_row_vec(&self, index: usize) -> Vec<NUM> {
        let mut ret = Vec::<NUM>::with_capacity(self.get_width());
        for i in 0..self.get_width() {
            ret.push(self.mat[index][i]);
        }
        ret
    }
}
impl<NUM: CharMathNumeric<NUM>> SquareMatrix<NUM, GenericMatrix<NUM>> for GenericMatrix<NUM> {}

pub mod matrices {
    use crate::linear::matrix::{GenericMatrix, Matrix, MatrixBase};
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
            *ret.get_value_ref_mut(i, i) = scales[i];
        }
        ret
    }
    pub fn translation<N: CharMathNumeric<N>>(size: usize, trans: &[N]) -> GenericMatrix<N> {
        let mut ret = identity::<N>(size);
        for i in 0..(size - 1usize) {
            *ret.get_value_ref_mut(size - 1usize, i) = trans[i];
        }
        ret
    }
}
