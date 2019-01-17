#[derive(Serialize, Deserialize, Debug)]
pub struct PolygonScene {
    pub name: String,
    pub scene_objects: Vec<SceneObject>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SceneObject {
    pub model_name: String,
    //pub position: ValueVec3,
    //pub euler_rotation: ValueVec3,
    //pub scale: ValueFloat,
    //pub layout_to_vars: Vec<UniformMapping>,
    //pub binding_to_buffers: Vec<BufferMapping>,
}
