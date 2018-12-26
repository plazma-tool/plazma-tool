extern crate intro_3d;

use intro_3d::{Vector3, Vector4, Matrix4, to_radians};

#[test]
fn add_vector3_and_vector3() {
    let res: Vector3 = Vector3::new(1.8, 4.2, 34.5) + Vector3::new(32.2, 12.4, 28.6);
    let expect = [34.0, 16.599998, 63.1];

    assert_eq!{format!{"{:?}", res.as_slice()}, format!{"{:?}", expect}};
}

#[test]
fn mul_matrix4_by_vector4() {
    let v: Vector4 = Vector4::new(1.8, 4.2, 34.5, 9.3);
    let m: Matrix4 = Matrix4::from_row_slice([[2.5, 3.4, 1.1, 5.2],
                                              [0.0, 1.0, 8.4, 8.9],
                                              [2.1, 6.7, 1.0, 4.4],
                                              [5.1, 0.0, 7.5, 2.1]]);

    let res = m * v;
    let expect = [105.09, 376.77, 107.34, 287.46];

    assert_eq!{format!{"{:?}", res.as_slice()}, format!{"{:?}", expect}};
}

#[test]
fn mul_matrix4_by_matrix4() {
    let a: Matrix4 = Matrix4::from_row_slice([[2.5, 3.4, 1.1, 5.2],
                                              [0.0, 1.0, 8.4, 8.9],
                                              [2.1, 6.7, 1.0, 4.4],
                                              [5.1, 0.0, 7.5, 2.1]]);

    let b: Matrix4 = Matrix4::from_row_slice([[3.8, 5.1, 7.2, 8.0],
                                              [4.2, 4.6, 2.6, 8.2],
                                              [6.3, 7.1, 8.5, 5.5],
                                              [5.4, 1.1, 0.0, 6.3]]);

    let res = a * b;
    // column major
    let expect = [[58.79,     105.17999, 66.17999,  77.97],
                  [41.920002, 74.03,     53.469997, 81.56999],
                  [36.190002, 73.99999,  41.039997, 100.47],
                  [86.69,     110.47,    104.96,    95.28]];

    assert_eq!{format!{"{:?}", res.as_column_slice()}, format!{"{:?}", expect}};
}

#[test]
fn look_at_lh_no_translate() {
    let target: Vector3 = Vector3::new(1.0, 2.0, 2.0);
    let up: Vector3 = Vector3::new(0.0, 1.0, 0.0);

    let res = Matrix4::look_at_lh_no_translate(&target, &up);
    // column major
    let expect = [[0.8944272,  -0.2981424, 0.33333334, 0.0],
                  [0.0,        0.745356,   0.6666667,  0.0],
                  [-0.4472136, -0.5962848, 0.6666667,  0.0],
                  [0.0,        0.0,        0.0,        1.0]];

    assert_eq!{format!{"{:?}", res.as_column_slice()}, format!{"{:?}", expect}};
}

#[test]
fn look_at_rh_no_translate() {
    let target: Vector3 = Vector3::new(1.0, 2.0, 2.0);
    let up: Vector3 = Vector3::new(0.0, 1.0, 0.0);

    let res = Matrix4::look_at_rh_no_translate(&target, &up);
    // column major
    let expect = [[-0.8944272, -0.2981424, -0.33333334, 0.0],
                  [0.0,        0.745356,   -0.6666667,  0.0],
                  [0.4472136,  -0.5962848, -0.6666667,  0.0],
                  [0.0,        0.0,        0.0,         1.0]];

    assert_eq!{format!{"{:?}", res.as_column_slice()}, format!{"{:?}", expect}};
}

#[test]
fn look_at_lh() {
    let position: Vector3 = Vector3::new(1.0, 2.0, 3.0);
    let target: Vector3 = Vector3::new(2.2, -3.4, 1.2);
    let up: Vector3 = Vector3::new(0.0, 1.0, 0.0);

    let res = Matrix4::look_at_lh(&position, &target, &up);
    // column major
    let expect = [[-0.83205026, 0.5149166,  0.20628424,  0.0],
                  [0.0,         0.3718842,  -0.92827904, 0.0],
                  [-0.5547002,  -0.7723749, -0.30942634, 0.0],
                  [2.496151,    1.0584399,  2.5785527,   1.0]];

    assert_eq!{format!{"{:?}", res.as_column_slice()}, format!{"{:?}", expect}};
}

#[test]
fn look_at_rh() {
    let position: Vector3 = Vector3::new(1.0, 2.0, 3.0);
    let target: Vector3 = Vector3::new(2.2, -3.4, 1.2);
    let up: Vector3 = Vector3::new(0.0, 1.0, 0.0);

    let res = Matrix4::look_at_rh(&position, &target, &up);
    // column major
    let expect = [[0.83205026, 0.5149166,  -0.20628424, 0.0],
                  [0.0,        0.3718842,  0.92827904,  0.0],
                  [0.5547002,  -0.7723749, 0.30942634,  0.0],
                  [-2.496151,  1.0584399,  -2.5785527,  1.0]];

    assert_eq!{format!{"{:?}", res.as_column_slice()}, format!{"{:?}", expect}};
}

#[test]
fn perspective_matrix() {
    let res = Matrix4::new_perspective(1024.0 / 768.0,
                                       45.0_f32.to_radians(),
                                       0.1,
                                       100.0);
    // column major
    let expect = [[1.81066, 0.0,       0.0,        0.0],
                  [0.0,     2.4142134, 0.0,        0.0],
                  [0.0,     0.0,       -1.002002,  -1.0],
                  [0.0,     0.0,       -0.2002002, 0.0]];

    assert_eq!{format!{"{:?}", res.as_column_slice()}, format!{"{:?}", expect}};
}

#[test]
fn convert_f32_to_radians() {
    let mut deg: f32 = 13.7;
    let mut res = to_radians(deg);
    let mut expect = 0.2391101;
    assert_eq!{res, expect}

    deg = -124.9;
    res = to_radians(deg);
    expect = -2.1799161;
    assert_eq!{res, expect}
}

#[test]
fn rotation_euler() {
    let res = Matrix4::new_rotation_euler(
        to_radians(12.0),
        to_radians(-25.2),
        to_radians(55.9)
    );

    let expect = [[0.5072813,    0.7492514,   0.42577928, 0.0],
                  [-0.8595956,   0.475084,    0.18812414, 0.0],
                  [-0.061328664, -0.46142983, 0.8850544,  0.0],
                  [0.0,          0.0,         0.0,        1.0]];

    assert_eq!{format!{"{:?}", res.as_column_slice()}, format!{"{:?}", expect}};
}

#[test]
fn add_two_vec3() {
    println!("");

    let a = Vector3::new(1.2, 4.5, -8.9);
    let b = Vector3::new(3.6, -8.5, 1.9);

    let res = a + b;
    let expect = [4.8, -4.0, -6.9999995];

    assert_eq!{format!{"{:?}", res.as_slice()}, format!{"{:?}", expect}};
}

#[test]
fn add_assign_vec3() {
    println!("");

    let mut res = Vector3::new(1.2, 4.5, -8.9);
    let b = Vector3::new(3.6, -8.5, 1.9);

    res += b;
    let expect = [4.8, -4.0, -6.9999995];

    assert_eq!{format!{"{:?}", res.as_slice()}, format!{"{:?}", expect}};
}
