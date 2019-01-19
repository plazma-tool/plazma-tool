use crate::dmo_data::mesh::Mesh;

#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    pub model_type: ModelType,
    pub meshes: Vec<Mesh>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ModelType {
    NOOP,
    Cube,
    Obj,
}
