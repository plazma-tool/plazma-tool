#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    pub name: String,
    pub model_type: ModelType,
    pub vert_src_path: String,
    pub frag_src_path: String,
    pub obj_path: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ModelType {
    NOOP,
    Cube,
    Obj,
}
