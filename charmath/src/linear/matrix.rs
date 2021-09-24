use crate::linear::vector::VectorBase;
use crate::numeric::CharMathNumeric;
use crate::CharMathCopy;

impl<NUM: CharMathNumeric<NUM>> CharMathCopy<Vec<Vec<NUM>>> for Vec<Vec<NUM>> {
    fn cm_copy(&self) -> Self {
        let mut ret = Vec::<Vec<NUM>>::with_capacity(self.len());
        for i in 0..self.len() {
            ret[i] = Vec::<NUM>::with_capacity(self[i].len());
            for j in 0..ret[i].len() {
                ret[i][j] = self[i][j];
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
}

pub trait Matrix<NUM: CharMathNumeric<NUM>, MAT: Matrix<NUM, MAT>>:
    MatrixBase<NUM> + CharMathCopy<MAT> //Algebraic<MAT, MAT>
{
    fn is_square(&self) -> bool {
        self.get_width() == self.get_height()
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
}
#[derive(Debug)]
pub struct GenericMatrix<NUM: CharMathNumeric<NUM>> {
    mat: Vec<Vec<NUM>>,
}
impl<NUM: CharMathNumeric<NUM>> GenericMatrix<NUM> {
    pub fn from_vec(vec: Vec<Vec<NUM>>) -> Self {
        GenericMatrix::<NUM> { mat: vec }
    }
    pub fn from_flat(arr: &[NUM], h: usize, w: usize) -> Self {
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
    pub fn sized(h: usize, w: usize) -> Self {
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
impl<NUM: CharMathNumeric<NUM>> Matrix<NUM, Self> for GenericMatrix<NUM> {}
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
