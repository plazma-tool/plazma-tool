use intro_3d::{Vector3, Matrix4};

pub struct Camera {
    pub fovy_angle: f32,
    pub aspect: f32,
    pub clip_near: f32,
    pub clip_far: f32,
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],

    pub position: Vector3,
    pub front: Vector3,
    pub up: Vector3,
    pub right: Vector3,
    pub world_up: Vector3,
    pub pitch: f32,
    pub yaw: f32,
}

impl Camera {
    /// If `front` vector is None, it will be set from pitch and yaw.
    pub fn new(fovy_angle: f32,
               aspect: f32,
               position: Vector3,
               front: Option<Vector3>,
               world_up: Vector3,
               pitch: f32,
               yaw: f32)
        -> Camera
        {
            let v = if let Some(ref a) = front {
                Vector3::new(a.x, a.y, a.z)
            } else {
                Vector3::new(0.0, 0.0, 0.0)
            };

        let mut camera = Camera {
            fovy_angle: fovy_angle,
            aspect: aspect,
            clip_near: 0.1,
            clip_far: 100.0,
            projection: [[0.0; 4]; 4],
            view: [[0.0; 4]; 4],
            world_up: world_up,
            position: position,
            front: v,
            up: Vector3::new(0.0, 0.0, 0.0),
            right: Vector3::new(0.0, 0.0, 0.0),
            pitch: pitch,
            yaw: yaw,
        };

        if let Some(_) = front {
            camera.calculate_right_and_up_from_front();
        } else {
            // pitch and yaw determines the front, right, up vector
            camera.do_pitch_and_yaw_from_mouse_delta(0.0, 0.0);
        }

        camera.update_projection();
        camera.update_view();

        camera
    }

    pub fn new_defaults(aspect: f32) -> Camera {
        Camera::new(45.0,
                    aspect,
                    Vector3::new(0.0, 0.0, 10.0),
                    None,
                    Vector3::new(0.0, 1.0, 0.0),
                    0.0,
                    90.0)
    }

    pub fn get_copy(&self) -> Camera {
        let position = self.get_position_copy();
        let front = self.get_front_copy();
        let world_up = self.get_world_up_copy();
        Camera::new(self.fovy_angle, self.aspect, position, Some(front), world_up, self.pitch, self.yaw)
    }

    pub fn update_projection(&mut self) {
        let a = Matrix4::new_perspective(self.aspect,
                                         self.fovy_angle.to_radians(),
                                         self.clip_near,
                                         self.clip_far);
        self.projection = a.as_column_slice();
    }

    pub fn do_pitch_and_yaw_from_mouse_delta(&mut self, dx: f32, dy: f32) {
        self.yaw += dx;
        self.pitch += dy;

        if self.pitch > 89.0_f32 {
            self.pitch = 89.0_f32;
        }
        if self.pitch < -89.0_f32 {
            self.pitch = -89.0_f32;
        }

        let x = self.pitch.to_radians().cos() * self.yaw.to_radians().cos();
        let y = self.pitch.to_radians().sin();
        let z = self.pitch.to_radians().cos() * self.yaw.to_radians().sin();

        self.front = Vector3::new(x, y, z).normalize();
        self.right = self.front.cross(&self.world_up).normalize();
        self.up = self.right.cross(&self.front).normalize();
    }

    pub fn calculate_right_and_up_from_front(&mut self) {
        self.right = self.front.cross(&self.world_up).normalize();
        self.up = self.right.cross(&self.front).normalize();
    }

    pub fn update_view(&mut self) {
        let a = Matrix4::look_at_rh(&self.position, &{&self.position + &self.front}, &self.up);
        self.view = a.as_row_slice();
    }

    pub fn move_forward(&mut self, speed: f32) {
        self.position += &self.front * speed;
    }

    pub fn move_backward(&mut self, speed: f32) {
        self.position -= &self.front * speed;
    }

    pub fn move_left(&mut self, speed: f32) {
        self.position -= self.front.cross(&self.up).normalize() * speed;
    }

    pub fn move_right(&mut self, speed: f32) {
        self.position += self.front.cross(&self.up).normalize() * speed;
    }

    pub fn get_position(&self) -> &Vector3 {
        &self.position
    }

    pub fn get_position_copy(&self) -> Vector3 {
        Vector3::new(self.position.x,
                     self.position.y,
                     self.position.z)
    }

    pub fn set_position(&mut self, position: Vector3) {
        self.position = position;
        self.update_view();
    }

    pub fn get_front(&self) -> &Vector3 {
        &self.front
    }

    pub fn get_front_copy(&self) -> Vector3 {
        Vector3::new(self.front.x,
                     self.front.y,
                     self.front.z)
    }

    pub fn get_world_up_copy(&self) -> Vector3 {
        Vector3::new(self.world_up.x,
                     self.world_up.y,
                     self.world_up.z)
    }

    pub fn set_front(&mut self, front: Vector3) {
        self.front = front;
        self.update_view();
    }
}
