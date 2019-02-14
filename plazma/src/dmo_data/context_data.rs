use std::path::PathBuf;
use std::error::Error;

use crate::utils::file_to_string;
use crate::dmo_data::quad_scene::QuadScene;
use crate::dmo_data::polygon_scene::PolygonScene;
use crate::dmo_data::polygon_context::PolygonContext;
use crate::dmo_data::data_index::DataIndex;
//use crate::dmo_data::polygon_context::PolygonContext;
use crate::dmo_data::sync_vars::SyncVars;

#[derive(Serialize, Deserialize, Debug)]
pub struct ContextData {
    pub quad_scenes: Vec<QuadScene>,
    pub polygon_scenes: Vec<PolygonScene>,
    pub polygon_context: PolygonContext,
    pub frame_buffers: Vec<FrameBuffer>,
    //pub audio_path: PathBuf,
    pub sync_vars: SyncVars,

    #[serde(skip_serializing, skip_deserializing)]
    pub index: DataIndex,
}

impl ContextData {
    pub fn read_shaders(&mut self) -> Result<(), Box<Error>> {
        for scene in self.quad_scenes.iter_mut() {
            if scene.vert_src_path.len() > 0 {
                scene.vert_src = file_to_string(&PathBuf::from(&scene.vert_src_path))?;
            }
            if scene.frag_src_path.len() > 0 {
                scene.frag_src = file_to_string(&PathBuf::from(&scene.frag_src_path))?;
            }
        }

        for model in self.polygon_context.models.iter_mut() {
            if model.vert_src_path.len() > 0 {
                model.vert_src = file_to_string(&PathBuf::from(&model.vert_src_path))?;
            }
            if model.frag_src_path.len() > 0 {
                model.frag_src = file_to_string(&PathBuf::from(&model.frag_src_path))?;
            }
        }

        Ok(())
    }

    pub fn build_index(&mut self) -> Result<(), Box<Error>> {
        // First, empty any existing index data.
        self.index = DataIndex::new();

        for (idx, scene) in self.quad_scenes.iter_mut().enumerate() {
            self.index.add_quad_scene(scene, idx)?;
        }

        for (idx, buffer) in self.frame_buffers.iter().enumerate() {
            self.index.add_frame_buffer(buffer, idx)?;
        }

        for (idx, model) in self.polygon_context.models.iter().enumerate() {
            self.index.add_model(model, idx)?;
        }

        for (idx, scene) in self.polygon_scenes.iter().enumerate() {
            self.index.add_polygon_scene(scene, idx)?;
        }

        Ok(())
    }
}

impl Default for ContextData {
    fn default() -> ContextData {
        ContextData {
            sync_vars: SyncVars::default(),
            quad_scenes: vec![],
            polygon_scenes: vec![],
            polygon_context: PolygonContext::default(),
            frame_buffers: vec![],
            //poly_models: vec![],
            //audio_path: PathBuf::from(""),
            index: DataIndex::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FrameBuffer {
    pub name: String,
    pub kind: BufferKind,
    pub format: PixelFormat,
    // image_data: ...
}

impl FrameBuffer {
    pub fn framebuffer_result_image() -> FrameBuffer {
        FrameBuffer{
            name: "RESULT_IMAGE".to_string(),
            kind: BufferKind::Empty_Texture,
            format: PixelFormat::RGBA_u8,
        }
    }
}

/// Specifies the frame buffer kind to be generated
#[derive(Serialize, Deserialize, Debug)]
pub enum BufferKind {
    NOOP,
    Empty_Texture,
    Image_Texture,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PixelFormat {
    NOOP,
    RED_u8,
    RGB_u8,
    RGBA_u8,
}

