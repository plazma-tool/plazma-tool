use crate::dmo_data::{UniformMapping, BufferMapping};

#[derive(Serialize, Deserialize, Debug)]
pub struct QuadScene {
    /// Will be mapped to `idx`, the array index in `quad_scenes[]`.
    pub name: String,
    /// Will be mapped to `vert_src_idx`, the array index in `shader_sources[]`.
    pub vert_src_path: String,
    /// Will be mapped to `frag_src_idx`, the array index in `shader_sources[]`.
    pub frag_src_path: String,

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
            layout_to_vars: vec![],
            binding_to_buffers: vec![],
        }
    }

    pub fn scene_draw_result() -> QuadScene {
        use crate::dmo_data::BuiltIn::*;
        QuadScene {
            name: "DRAW_RESULT".to_string(),
            // FIXME use static strings, b/c these will always need to be present when executing the binary
            vert_src_path: "./data/builtin/screen_quad.vert".to_string(),
            frag_src_path: "./data/builtin/draw_result.frag".to_string(),
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

// const DRAW_RESULT_VERT_SRC: &'static str = include_str!("../../data/builtin/screen_quad.vert");
//
// const DRAW_RESULT_FRAG_SRC: &'static str = include_str!("../../data/builtin/draw_result.frag");
