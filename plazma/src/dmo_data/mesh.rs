#[derive(Serialize, Deserialize, Debug)]
pub struct Mesh {
    pub name: String,
    pub vert_src_path: String,
    pub vert_src: String,
    pub frag_src_path: String,
    pub frag_src: String,

    pub obj_path: Option<String>,

    //pub textures: Vec<Texture>,
}
