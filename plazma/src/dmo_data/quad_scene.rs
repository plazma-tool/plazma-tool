use crate::dmo_data::{UniformMapping, BufferMapping};

#[derive(Serialize, Deserialize, Debug)]
pub struct QuadScene {
    pub name: String,
    pub vert_src_path: String,
    pub vert_src: String,
    pub frag_src_path: String,
    pub frag_src: String,

    /// Which index in `ContextData.sync_vars[]` corresponds to a uniform layout
    /// binding in the fragment shader.
    pub layout_to_vars: Vec<UniformMapping>,

    /// Which index in `ContextData.frame_buffers[]` corresponds to a texture
    /// binding in the fragment shader.
    pub binding_to_buffers: Vec<BufferMapping>,
}

impl Default for QuadScene {
    fn default() -> QuadScene {
        QuadScene::circle()
    }

}

impl QuadScene {
    pub fn circle() -> QuadScene {
        // FIXME these will have to be relative to project root stored in ProjectData
        let vert_src_path = "../data/screen_quad.vert".to_string();
        let vert_src = include_str!("../../data/screen_quad.vert").to_string();
        let frag_src_path = "../data/circle.frag".to_string();
        let frag_src = include_str!("../../data/circle.frag").to_string();

        QuadScene {
            name: "default".to_string(),
            vert_src_path: vert_src_path,
            vert_src: vert_src,
            frag_src_path: frag_src_path,
            frag_src: frag_src,
            layout_to_vars: vec![
                UniformMapping::Float(0,
                                      "time".to_string()),
                UniformMapping::Vec2(1,
                                     "window_resolution.x".to_string(),
                                     "window_resolution.y".to_string()),
                UniformMapping::Vec2(3,
                                     "screen_resolution.x".to_string(),
                                     "screen_resolution.y".to_string()),
            ],
            binding_to_buffers: vec![],
        }
    }

    pub fn cross() -> QuadScene {
        // FIXME these will have to be relative to project root stored in ProjectData
        let vert_src_path = "../data/screen_quad.vert".to_string();
        let vert_src = include_str!("../../data/screen_quad.vert").to_string();
        let frag_src_path = "../data/cross.frag".to_string();
        let frag_src = include_str!("../../data/cross.frag").to_string();

        QuadScene {
            name: "default".to_string(),
            vert_src_path: vert_src_path,
            vert_src: vert_src,
            frag_src_path: frag_src_path,
            frag_src: frag_src,
            layout_to_vars: vec![
                UniformMapping::Float(0,
                                      "time".to_string()),
                UniformMapping::Vec2(1,
                                     "window_resolution.x".to_string(),
                                     "window_resolution.y".to_string()),
                UniformMapping::Vec2(3,
                                     "screen_resolution.x".to_string(),
                                     "screen_resolution.y".to_string()),
            ],
            binding_to_buffers: vec![],
        }
    }
}
