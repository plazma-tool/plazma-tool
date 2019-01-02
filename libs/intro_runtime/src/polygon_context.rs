use smallvec::SmallVec;

use intro_3d::{Vector3, Matrix4, to_radians};

use crate::model::Model;

pub struct PolygonContext {
    pub view_position: Vector3,
    pub view_front: Vector3,
    pub view_up: Vector3,

    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4],

    pub view_position_var_idx: [usize; 3],
    pub view_front_var_idx: [usize; 3],

    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    pub models: SmallVec<[Model; 4]>,
}

impl Default for PolygonContext {
    fn default() -> PolygonContext {
        PolygonContext {
            view_position: Vector3::from_slice(&[0.0; 3]),
            view_front: Vector3::from_slice(&[0.0; 3]),
            view_up: Vector3::new(0.0, 1.0, 0.0),

            view_matrix: [[0.0; 4]; 4],
            projection_matrix: [[0.0; 4]; 4],

            view_position_var_idx: [0; 3],
            view_front_var_idx: [0; 3],

            fovy: 0.0,
            znear: 0.0,
            zfar: 0.0,

            models: SmallVec::new(),
        }
    }
}

impl PolygonContext {
    pub fn update_view_matrix(&mut self) {
        let m = Matrix4::look_at_rh(&self.view_position,
                                    &{&self.view_position + &self.view_front},
                                    &self.view_up);
        self.view_matrix = m.as_column_slice();
    }

    pub fn update_projection_matrix(&mut self, aspect: f32) {
        let a = Matrix4::new_perspective(aspect,
                                         to_radians(self.fovy),
                                         self.znear,
                                         self.zfar);
        self.projection_matrix = a.as_column_slice();
    }
}

