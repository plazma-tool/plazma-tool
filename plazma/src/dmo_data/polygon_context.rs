use crate::dmo_data::model::Model;

// FIXME use ValueVec3 to sync position, front and up

#[derive(Serialize, Deserialize, Debug)]
pub struct PolygonContext {
    pub view_position: [f32; 3],
    pub view_front: [f32; 3],
    pub view_up: [f32; 3],

    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    pub models: Vec<Model>,
}

impl Default for PolygonContext {
    fn default() -> PolygonContext {
        PolygonContext {
            view_position: [0.0; 3],
            view_front: [0.0; 3],
            view_up: [0.0; 3],
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            models: vec![],
        }
    }
}
