use smallvec::SmallVec;

use intro_3d::{Vector3, Matrix4, to_radians};

use crate::model::Model;

pub struct PolygonContext {
    pub view_position: Vector3,
    pub view_front: Vector3,
    pub view_up: Vector3,

    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4],

    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    pub models: SmallVec<[Model; 4]>,
}

impl Default for PolygonContext {
    fn default() -> PolygonContext {
        PolygonContext::new_defaults(1.7777)// 16:9 aspect
    }
}

impl PolygonContext {
    pub fn new_defaults(aspect: f32) -> PolygonContext {
        PolygonContext::new(
            Vector3::from_slice(&[0.0; 3]),
            Vector3::from_slice(&[0.0; 3]),
            Vector3::new(0.0, 1.0, 0.0),
            45.0,
            0.1,
            100.0,
            aspect
        )
    }

    pub fn new(view_position: Vector3,
               view_front: Vector3,
               view_up: Vector3,
               fovy: f32,
               znear: f32,
               zfar: f32,
               aspect: f32)
               -> PolygonContext
    {
        let mut p = PolygonContext {
            view_position: view_position,
            view_front: view_front,
            view_up: view_up,

            view_matrix: [[0.0; 4]; 4],
            projection_matrix: [[0.0; 4]; 4],

            fovy: fovy,
            znear: znear,
            zfar: zfar,

            models: SmallVec::new(),
        };

        p.update_view_matrix();
        p.update_projection_matrix(aspect);

        p
    }

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

    pub fn get_view_position(&self) -> &Vector3 {
        &self.view_position
    }

    pub fn get_view_front(&self) -> &Vector3 {
        &self.view_front
    }
}

