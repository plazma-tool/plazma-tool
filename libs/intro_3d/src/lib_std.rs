// Lots of stuff learned from http://nalgebra.org/

// The homogeneous transformation matrix for 3D bodies
// http://planning.cs.uiuc.edu/node104.html

// http://mathworld.wolfram.com/RotationMatrix.html

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

pub const PI: f64 = 3.14159265358979323846;
pub const DEG_TO_RAD: f64 = 0.01745329251994329577;

// === Vector3 =================================================================

pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x: x, y: y, z: z }
    }

    pub fn from_slice(d: &[f32; 3]) -> Vector3 {
        Vector3 {
            x: d[0],
            y: d[1],
            z: d[2],
        }
    }

    pub fn from_vec(v: &Vector3) -> Vector3 {
        Vector3 {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }

    pub fn as_slice(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    pub fn clone(&self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }

    /// Length.
    pub fn norm(&self) -> f32 {
        f32::sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
    }

    pub fn normalize(&self) -> Vector3 {
        let a = Vector3::from_vec(self);
        a / self.norm()
    }

    pub fn cross(&self, b: &Vector3) -> Vector3 {
        let ax = self.x;
        let ay = self.y;
        let az = self.z;

        let bx = b.x;
        let by = b.y;
        let bz = b.z;

        Vector3::new(ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx)
    }
}

// === Vector4 =================================================================

pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vector4 {
        Vector4 {
            x: x,
            y: y,
            z: z,
            w: w,
        }
    }

    pub fn from_slice(d: &[f32; 4]) -> Vector4 {
        Vector4 {
            x: d[0],
            y: d[1],
            z: d[2],
            w: d[3],
        }
    }

    pub fn as_slice(&self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }
}

// === Matrix4 =================================================================

pub struct Matrix4 {
    data: [[f32; 4]; 4],
}

impl Matrix4 {
    /// Returns an identity matrix.
    pub fn new() -> Matrix4 {
        Matrix4::identity()
    }

    pub fn identity() -> Matrix4 {
        Matrix4 {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn zero() -> Matrix4 {
        Matrix4 {
            data: [[0.0; 4]; 4],
        }
    }

    pub fn from_row_slice(data: [[f32; 4]; 4]) -> Matrix4 {
        Matrix4 { data: data }
    }

    pub fn as_slice(&self) -> [[f32; 4]; 4] {
        self.as_row_slice()
    }

    pub fn as_row_slice(&self) -> [[f32; 4]; 4] {
        self.data
    }

    pub fn as_column_slice(&self) -> [[f32; 4]; 4] {
        let mut columns: [[f32; 4]; 4] = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                columns[i][j] = self.data[j][i];
            }
        }

        columns
    }

    pub fn new_translation(v: &Vector3) -> Matrix4 {
        Matrix4 {
            data: [
                [1.0, 0.0, 0.0, v.x],
                [0.0, 1.0, 0.0, v.y],
                [0.0, 0.0, 1.0, v.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Creates a new rotation from Euler angles.
    ///
    /// The primitive rotations are applied in order: 1 roll − 2 pitch − 3 yaw.
    pub fn new_rotation_euler(roll: f32, pitch: f32, yaw: f32) -> Matrix4 {
        let (sr, cr) = (f32::sin(roll), f32::cos(roll));
        let (sp, cp) = (f32::sin(pitch), f32::cos(pitch));
        let (sy, cy) = (f32::sin(yaw), f32::cos(yaw));

        let data: [[f32; 4]; 4] = [
            [cy * cp, cy * sp * sr - sy * cr, cy * sp * cr + sy * sr, 0.0],
            [sy * cp, sy * sp * sr + cy * cr, sy * sp * cr - cy * sr, 0.0],
            [-sp, cp * sr, cp * cr, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        Matrix4::from_row_slice(data)
    }

    pub fn set_translation(&mut self, v: &Vector3) {
        self.data[0][3] = v.x;
        self.data[1][3] = v.y;
        self.data[2][3] = v.z;
    }

    pub fn apply_translation(&mut self, v: &Vector3) {
        self.data[0][3] += v.x;
        self.data[1][3] += v.y;
        self.data[2][3] += v.z;
    }

    pub fn set_scale(&mut self, v: &Vector3) {
        self.data[0][0] = v.x;
        self.data[1][1] = v.y;
        self.data[2][2] = v.z;
    }

    pub fn apply_scale(&mut self, v: &Vector3) {
        self.data[0][0] *= v.x;
        self.data[1][1] *= v.y;
        self.data[2][2] *= v.z;
    }

    pub fn rotate_euler(&mut self, roll: f32, pitch: f32, yaw: f32) {
        let a = Matrix4::from_row_slice(self.data);
        *self = a * Matrix4::new_rotation_euler(roll, pitch, yaw);
    }

    pub fn inverse(&self) -> Matrix4 {
        self.transpose()
    }

    pub fn transpose(&self) -> Matrix4 {
        let mut data: [[f32; 4]; 4] = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                data[i][j] = self.data[j][i];
            }
        }

        Matrix4::from_row_slice(data)
    }

    /// Creates a rotation that corresponds to the local frame of an observer standing at the
    /// origin and looking toward `dir`.
    ///
    /// It maps the view direction `dir` to the positive `z` axis.
    pub fn new_observer_frame(dir: &Vector3, up: &Vector3) -> Matrix4 {
        let zaxis = dir.normalize();
        let xaxis = up.cross(&zaxis).normalize();
        let yaxis = zaxis.cross(&xaxis).normalize();

        let data: [[f32; 4]; 4] = [
            [xaxis.x, yaxis.x, zaxis.x, 0.0],
            [xaxis.y, yaxis.y, zaxis.y, 0.0],
            [xaxis.z, yaxis.z, zaxis.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        Matrix4::from_row_slice(data)
    }

    pub fn look_at_rh_no_translate(dir: &Vector3, up: &Vector3) -> Matrix4 {
        Matrix4::new_observer_frame(&dir.neg(), up).inverse()
    }

    pub fn look_at_lh_no_translate(dir: &Vector3, up: &Vector3) -> Matrix4 {
        Matrix4::new_observer_frame(dir, up).inverse()
    }

    /// Builds a right-handed look-at view matrix.
    pub fn look_at_rh(eye: &Vector3, target: &Vector3, up: &Vector3) -> Matrix4 {
        let rotation = Matrix4::look_at_rh_no_translate(&(target - eye), up);

        // FIXME Simplify this. It is only because the rotation matrix expects a
        // Vec4 but the eye is Vec3.
        let m_rotation = Matrix4::from_row_slice(rotation.data);
        let v_eye = Vector4::new(eye.x, eye.y, eye.z, 0.0);
        let v_res = m_rotation * (-v_eye);
        let v_tr = Vector3::new(v_res.x, v_res.y, v_res.z);

        let translation = Matrix4::new_translation(&v_tr);

        (translation * rotation)
    }

    /// Builds a left-handed look-at view matrix.
    pub fn look_at_lh(eye: &Vector3, target: &Vector3, up: &Vector3) -> Matrix4 {
        let rotation = Matrix4::look_at_lh_no_translate(&(target - eye), up);

        // FIXME Simplify this. It is only because the rotation matrix expects a
        // Vec4 but the eye is Vec3.
        let m_rotation = Matrix4::from_row_slice(rotation.data);
        let v_eye = Vector4::new(eye.x, eye.y, eye.z, 0.0);
        let v_res = m_rotation * (-v_eye);
        let v_tr = Vector3::new(v_res.x, v_res.y, v_res.z);

        let translation = Matrix4::new_translation(&v_tr);

        (translation * rotation)
    }

    /// Updates this perspective with a new y field of view of the view frustrum.
    pub fn set_fovy(&mut self, fovy: f32) {
        let old_m22 = self.data[1][1];
        self.data[1][1] = 1.0 / tanf32(fovy);
        self.data[0][0] = self.data[0][0] * (self.data[1][1] / old_m22);
    }

    /// Updates this perspective matrix with a new `width / height` aspect ratio of the view
    /// frustrum.
    pub fn set_aspect(&mut self, aspect: f32) {
        if aspect == 0.0 {
            panic! {"Aspect ratio must not be zero."};
        }
        self.data[0][0] = self.data[1][1] / aspect;
    }

    /// Updates this perspective matrix with new near and far plane offsets of the view frustrum.
    pub fn set_znear_and_zfar(&mut self, znear: f32, zfar: f32) {
        self.data[2][2] = (zfar + znear) / (znear - zfar);
        self.data[2][3] = (zfar * znear * 2.0) / (znear - zfar);
    }

    pub fn new_perspective(aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Matrix4 {
        if aspect == 0.0 {
            panic! {"Aspect ratio must not be zero."};
        }
        if zfar - znear == 0.0 {
            panic! {"The near-plane and far-plane must not be superimposed."};
        }

        let mut res = Matrix4::identity();
        res.set_fovy(fovy);
        res.set_aspect(aspect);
        res.set_znear_and_zfar(znear, zfar);

        res.data[3][3] = 0.0;
        res.data[3][2] = -1.0;

        res
    }

    pub fn new_homogeneous(translation: &Vector3, euler_rotation: &Vector3, scale: f32) -> Matrix4 {
        let mut m = Matrix4::identity();
        m.apply_scale(&Vector3::new(scale, scale, scale));
        m.rotate_euler(euler_rotation.x, euler_rotation.y, euler_rotation.z);
        m.apply_translation(translation);

        m
    }
}

// === Ops for Vector3 =========================================================

impl Mul<f32> for Vector3 {
    type Output = Vector3;
    fn mul(self, rhs: f32) -> Vector3 {
        Vector3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl<'a> Mul<f32> for &'a Vector3 {
    type Output = Vector3;
    fn mul(self, rhs: f32) -> Vector3 {
        Vector3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl MulAssign<f32> for Vector3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<f32> for Vector3 {
    type Output = Vector3;
    fn div(self, rhs: f32) -> Vector3 {
        Vector3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl DivAssign<f32> for Vector3 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Add<Vector3> for Vector3 {
    type Output = Vector3;
    fn add(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<'a> Add<&'a Vector3> for &'a Vector3 {
    type Output = Vector3;
    fn add(self, rhs: &Vector3) -> Vector3 {
        Vector3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign<Vector3> for Vector3 {
    fn add_assign(&mut self, rhs: Vector3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub<Vector3> for Vector3 {
    type Output = Vector3;
    fn sub(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign<Vector3> for Vector3 {
    fn sub_assign(&mut self, rhs: Vector3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<'a> Sub<&'a Vector3> for &'a Vector3 {
    type Output = Vector3;
    fn sub(self, rhs: &Vector3) -> Vector3 {
        let a = self;
        let b = rhs;
        Vector3::new(a.x - b.x, a.y - b.y, a.z - b.z)
    }
}

impl Neg for Vector3 {
    type Output = Vector3;
    fn neg(self) -> Vector3 {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}

impl<'a> Neg for &'a Vector3 {
    type Output = Vector3;
    fn neg(self) -> Vector3 {
        let v = self;
        Vector3::new(-v.x, -v.y, -v.z)
    }
}

// === Ops for Vector4 =========================================================

impl Add<Vector4> for Vector4 {
    type Output = Vector4;

    fn add(self, rhs: Vector4) -> Vector4 {
        Vector4::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

impl Neg for Vector4 {
    type Output = Vector4;
    fn neg(self) -> Vector4 {
        Vector4::new(-self.x, -self.y, -self.z, -self.w)
    }
}

impl<'a> Neg for &'a Vector4 {
    type Output = Vector4;
    fn neg(self) -> Vector4 {
        let v = self;
        Vector4::new(-v.x, -v.y, -v.z, -v.w)
    }
}

// === Ops for Matrix4 =========================================================

impl Add<Matrix4> for Matrix4 {
    type Output = Matrix4;
    fn add(self, rhs: Matrix4) -> Matrix4 {
        let a = self.data;
        let b = rhs.data;
        Matrix4 {
            data: [
                [
                    a[0][0] + b[0][0],
                    a[0][1] + b[0][1],
                    a[0][2] + b[0][2],
                    a[0][3] + b[0][3],
                ],
                [
                    a[1][0] + b[1][0],
                    a[1][1] + b[1][1],
                    a[1][2] + b[1][2],
                    a[1][3] + b[1][3],
                ],
                [
                    a[2][0] + b[2][0],
                    a[2][1] + b[2][1],
                    a[2][2] + b[2][2],
                    a[2][3] + b[2][3],
                ],
                [
                    a[3][0] + b[3][0],
                    a[3][1] + b[3][1],
                    a[3][2] + b[3][2],
                    a[3][3] + b[3][3],
                ],
            ],
        }
    }
}

impl Mul<Matrix4> for Matrix4 {
    type Output = Matrix4;
    fn mul(self, rhs: Matrix4) -> Matrix4 {
        let a = self.data;
        let b = rhs.data;

        let mut c: [[f32; 4]; 4] = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    c[i][j] += a[i][k] * b[k][j];
                }
            }
        }

        Matrix4::from_row_slice(c)
    }
}

impl<'a> Mul<&'a Matrix4> for &'a Matrix4 {
    type Output = Matrix4;
    fn mul(self, rhs: &Matrix4) -> Matrix4 {
        let a = self.data;
        let b = rhs.data;

        let mut c: [[f32; 4]; 4] = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    c[i][j] += a[i][k] * b[k][j];
                }
            }
        }

        Matrix4::from_row_slice(c)
    }
}

impl Mul<Vector4> for Matrix4 {
    type Output = Vector4;
    fn mul(self, rhs: Vector4) -> Vector4 {
        let a = self.as_slice();
        let b = rhs.as_slice();

        let mut c: [f32; 4] = [0.0; 4];

        for i in 0..4 {
            for k in 0..4 {
                c[i] += a[i][k] * b[k];
            }
        }

        Vector4::from_slice(&c)
    }
}

impl Mul<f32> for Matrix4 {
    type Output = Matrix4;
    fn mul(self, rhs: f32) -> Matrix4 {
        let left = self.data;
        Matrix4 {
            data: [
                [
                    left[0][0] * rhs,
                    left[0][1] * rhs,
                    left[0][2] * rhs,
                    left[0][3] * rhs,
                ],
                [
                    left[1][0] * rhs,
                    left[1][1] * rhs,
                    left[1][2] * rhs,
                    left[1][3] * rhs,
                ],
                [
                    left[2][0] * rhs,
                    left[2][1] * rhs,
                    left[2][2] * rhs,
                    left[2][3] * rhs,
                ],
                [
                    left[3][0] * rhs,
                    left[3][1] * rhs,
                    left[3][2] * rhs,
                    left[3][3] * rhs,
                ],
            ],
        }
    }
}

// === Helper functions ========================================================

/// Calculates tangent from sin cos, no direct function in core::intrinsics.
pub fn tanf32(a: f32) -> f32 {
    let x: f64 = f64::sin(a as f64 / 2.0) / f64::cos(a as f64 / 2.0);
    x as f32
}

pub fn to_radians(degree: f32) -> f32 {
    ((degree as f64) * DEG_TO_RAD) as f32
}
