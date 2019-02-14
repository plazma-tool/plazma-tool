#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    pub name: String,
    pub model_type: ModelType,
    pub vert_src_path: String,
    pub frag_src_path: String,
    pub obj_path: String,

    #[serde(skip_serializing, skip_deserializing)]
    pub vert_src: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub frag_src: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ModelType {
    NOOP,
    Cube,
    Obj,
}
