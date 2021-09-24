#[cfg(test)]
mod tests {
    use charmath::linear::matrix::*;
    use charmath::linear::vector::*;

    #[test]
    fn base_test() {
        let a = Vec3D::new(3f64, 1f64, 4f64);
        let b = Vec2D::new_vec(&a);
        let c = Vec4D::new_vec(&b);
        let d = &a * Vec3D::new(2f64, -1f64, 0.5f64);
        println!(
            "VECTORS:\n\ta: {:?}\n\tb: {:?}\n\tc: {:?}\n\td: {:?}",
            a, b, c, d
        );
        let e = GenericMatrix::<f64>::from_flat(
            &[1f64, 0f64, 0f64, 1f64, 1f64, 0f64, 0f64, 0f64, 1f64],
            3,
            3,
        );
        let f = e.mul_col_vec(&a);
        let g = e.mul_row_vec(&a);
        println!(
            "MATRICES:\n\te: {:?}\n\tf (col): {:?}\n\tg (row): {:?}",
            e, f, g
        );
        panic!();
    }
}
