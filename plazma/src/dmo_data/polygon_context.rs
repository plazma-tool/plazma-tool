use crate::dmo_data::model::Model;

pub struct PolygonContext {
    // pub view_position: Vector3,
    // pub view_front: Vector3,
    // pub view_up: Vector3,

    // pub fovy: f32,
    // pub znear: f32,
    // pub zfar: f32,

    pub models: Vec<Model>,
}

impl Default for PolygonContext {
    fn default() -> PolygonContext {
        PolygonContext {
            models: vec![],
        }
    }
}
