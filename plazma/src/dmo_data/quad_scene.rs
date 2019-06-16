use crate::dmo_data::{BufferMapping, UniformMapping};

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
            // shader name here is only for index mapping, not going to read it as a path
            vert_src_path: DRAW_RESULT_VERT_SRC_PATH.to_string(),
            frag_src_path: DRAW_RESULT_FRAG_SRC_PATH.to_string(),
            layout_to_vars: vec![
                UniformMapping::Vec2(0, Window_Width, Window_Height),
                UniformMapping::Vec2(1, Screen_Width, Screen_Height),
            ],
            binding_to_buffers: vec![BufferMapping::Sampler2D(0, "RESULT_IMAGE".to_string())],
        }
    }
}

pub const DRAW_RESULT_VERT_SRC_PATH: &str = "data_builtin_screen_quad.vert";
pub const DRAW_RESULT_FRAG_SRC_PATH: &str = "data_builtin_draw_result.frag";
