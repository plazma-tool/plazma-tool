use crate::dmo_data::{UniformMapping, BufferMapping};

#[derive(Serialize, Deserialize, Debug)]
pub struct QuadScene {
    pub name: String,
    pub vert_src_path: String,
    pub frag_src_path: String,

    #[serde(skip_serializing, skip_deserializing)]
    pub vert_src: String,
    #[serde(skip_serializing, skip_deserializing)]
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
        QuadScene::new()
    }
}

impl QuadScene {
    pub fn new() -> QuadScene {
        QuadScene {
            name: "".to_string(),
            vert_src_path: "".to_string(),
            frag_src_path: "".to_string(),
            vert_src: "".to_string(),
            frag_src: "".to_string(),
            layout_to_vars: vec![],
            binding_to_buffers: vec![],
        }
    }

    pub fn scene_draw_result() -> QuadScene {
        use crate::dmo_data::BuiltIn::*;
        QuadScene {
            name: "DRAW_RESULT".to_string(),
            vert_src_path: "".to_string(),
            frag_src_path: "".to_string(),
            vert_src: DRAW_RESULT_VERT_SRC.to_string(),
            frag_src: DRAW_RESULT_FRAG_SRC.to_string(),
            layout_to_vars: vec![
                UniformMapping::Vec2(0, Window_Width, Window_Height),
                UniformMapping::Vec2(1, Screen_Width, Screen_Height),
            ],
            binding_to_buffers: vec![
                BufferMapping::Sampler2D(0, "RESULT_IMAGE".to_string()),
            ],
        }
    }
}

const DRAW_RESULT_VERT_SRC: &'static str = include_str!("../../data/builtin/screen_quad.vert");

const DRAW_RESULT_FRAG_SRC: &'static str = include_str!("../../data/builtin/draw_result.frag");

