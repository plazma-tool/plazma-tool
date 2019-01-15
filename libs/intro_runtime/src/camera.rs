use intro_3d::{Vector3, Matrix4};

pub struct Camera {
    pub fovy_angle: f32,
    pub aspect: f32,
    pub clip_near: f32,
    pub clip_far: f32,
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],

    position: Vector3,
    front: Vector3,
    pub up: Vector3,
    pub right: Vector3,
    pub world_up: Vector3,
    pub pitch: f32,
    pub yaw: f32,
}

impl Camera {
    pub fn new(fovy_angle: f32, aspect: f32, position: Vector3, world_up: Vector3, pitch: f32, yaw: f32) -> Camera {

        let mut camera = Camera {
            fovy_angle: fovy_angle,
            aspect: aspect,
            clip_near: 0.1,
            clip_far: 100.0,
            projection: [[0.0; 4]; 4],
            view: [[0.0; 4]; 4],
            world_up: world_up,
            position: position,
            front: Vector3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 0.0, 0.0),
            right: Vector3::new(0.0, 0.0, 0.0),
            pitch: pitch,
            yaw: yaw,
        };

        // pitch and yaw determines the front, right, up vector
        camera.do_pitch_and_yaw_from_mouse_delta(0.0, 0.0);
        camera.update_projection();
        camera.update_view();

        camera
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

    pub fn set_position(&mut self, position: Vector3) {
        self.position = position;
        self.update_view();
    }

    pub fn get_front(&self) -> &Vector3 {
        &self.front
    }

    pub fn set_front(&mut self, front: Vector3) {
        self.front = front;
        self.update_view();
    }
}
