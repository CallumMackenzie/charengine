#[cfg(test)]
mod tests {
    use charmath::linear::matrix::*;
    use charmath::linear::vector::*;

    #[test]
    fn vec2_create() {
        let a_x = 3.3;
        let a_y = 1.4;
        let a_len = f64::sqrt(a_x * a_x + a_y * a_y);
        let a = Vec2D::new(a_x, a_y);
        assert_eq!(a.get_x(), a_x, "Vec2D x val wrong.");
        assert_eq!(a.get_y(), a_y, "Vec2D y val wrong.");
        assert_eq!(a.len(), a_len, "Vec2D len wrong.");
        let a = Vec2D::new_arr(&[a_x, a_y]);
        assert_eq!(a.get_x(), a_x, "Vec2D arr x val wrong.");
        assert_eq!(a.get_y(), a_y, "Vec2D arr y val wrong.");
        assert_eq!(a.len(), a_len, "Vec2D arr len wrong.");
    }

    #[test]
    fn vec2_math() {
        let (a_x, a_y) = (1223.3233, 883.323219);
        let (b_x, b_y) = (323.32, -0.233133);
        let a_len = f64::sqrt(a_x * a_x + a_y * a_y);
        let a = Vec2D::new(a_x, a_y);
        let b = Vec2D::new(b_x, b_y);
        assert_eq!(
            a.normalized().get_x(),
            a_x / a_len,
            "Vec2D normalized x val wrong."
        );
        assert_eq!(
            a.normalized().get_y(),
            a_y / a_len,
            "Vec2D normalized y val wrong."
        );
        assert_eq!(a.add_num(2.0).get_x(), a_x + 2.0, "Add num x wrong.");
        assert_eq!(a.add_num(1.1).get_y(), a_y + 1.1, "Add num y wrong");
        assert_eq!(a.sub_num(8.1).get_x(), a_x - 8.1, "Sub num x wrong.");
        assert_eq!(a.sub_num(0.11).get_y(), a_y - 0.11, "Sub num y wrong");
        assert_eq!(a.mul_num(0.44).get_x(), a_x * 0.44, "Mul num x wrong.");
        assert_eq!(a.mul_num(-12.3).get_y(), a_y * -12.3, "Mul num y wrong");
        assert_eq!(a.div_num(3.32).get_x(), a_x / 3.32, "Div num x wrong.");
        assert_eq!(a.div_num(2323.1).get_y(), a_y / 2323.1, "Div num y wrong");

        assert_eq!(a.add_vec(&b).get_x(), a_x + b_x, "Add vec x wrong.");
        assert_eq!(a.add_vec(&b).get_y(), a_y + b_y, "Add vec y wrong");
        assert_eq!(a.sub_vec(&b).get_x(), a_x - b_x, "Sub vec x wrong.");
        assert_eq!(a.sub_vec(&b).get_y(), a_y - b_y, "Sub vec y wrong");
        assert_eq!(a.mul_vec(&b).get_x(), a_x * b_x, "Mul vec x wrong.");
        assert_eq!(a.mul_vec(&b).get_y(), a_y * b_y, "Mul vec y wrong");
        assert_eq!(a.div_vec(&b).get_x(), a_x / b_x, "Div vec x wrong.");
        assert_eq!(a.div_vec(&b).get_y(), a_y / b_y, "Div vec y wrong");
    }

    #[test]
    fn generc_matrix_create() {
        let mat_arr = [1.0, 9.0, 3.3, 0.2, 1.4, 3.41];
        let mat_w = 2;
        let mat_h = 3;
        let mat = GenericMatrix::<f64>::from_flat(&mat_arr, mat_h, mat_w);
        assert_eq!(mat.get_width(), mat_w, "Matrix wid wrong.");
        assert_eq!(mat.get_height(), mat_h, "Matrix hei wrong.");
    }

    #[test]
    fn matrix_utils() {
        let t3d = matrices::translation_3d::<f64, Vec3D>(&Vec3D::new(2f64, 3f64, 8f64));
        println!("Translation 3D: {:?}", t3d);
        let s3d = matrices::scale_3d::<f64, Vec3D>(&Vec3D::new(3.3f64, 0.4f64, 1.3f64));
        println!("Scale 3D: {:?}", s3d);
        let r3d = matrices::rotation_euler(1f64, 0.3f64, -0.1f64);
        println!("Euler 3D: {:?}", r3d);
        let a3d = s3d.mul_mat(&r3d).mul_mat(&t3d);
        println!("Composited: {:?}", a3d);
        let origin = Vec4D::new(0f64, 0f64, 0f64, 1f64);
        println!("New origin pos: {:?}", a3d.mul_row_vec(&origin));
    }
}
