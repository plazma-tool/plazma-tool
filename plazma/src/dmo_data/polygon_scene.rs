use crate::dmo_data::{BufferMapping, UniformMapping, ValueFloat, ValueVec3};

#[derive(Serialize, Deserialize, Debug)]
pub struct PolygonScene {
    pub name: String,
    pub scene_objects: Vec<SceneObject>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SceneObject {
    pub name: String,
    pub position: ValueVec3,
    pub euler_rotation: ValueVec3,
    pub scale: ValueFloat,
    pub layout_to_vars: Vec<UniformMapping>,
    pub binding_to_buffers: Vec<BufferMapping>,
}

impl Default for PolygonScene {
    fn default() -> PolygonScene {
        PolygonScene::empty()
    }
}

impl PolygonScene {
    pub fn empty() -> PolygonScene {
        PolygonScene {
            name: "empty".to_string(),
            scene_objects: vec![],
        }
    }
}
