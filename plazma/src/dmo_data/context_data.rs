use std::path::PathBuf;
use std::error::Error;

use crate::utils::file_to_string;
use crate::dmo_data::quad_scene::QuadScene;
use crate::dmo_data::polygon_scene::PolygonScene;
use crate::dmo_data::data_index::DataIndex;
//use crate::dmo_data::polygon_context::PolygonContext;
use crate::dmo_data::sync_vars::SyncVars;

#[derive(Serialize, Deserialize, Debug)]
pub struct ContextData {
    pub quad_scenes: Vec<QuadScene>,
    pub polygon_scenes: Vec<PolygonScene>,
    //pub polygon_context: PolygonContext,
    pub frame_buffers: Vec<FrameBuffer>,
    //pub audio_path: PathBuf,
    pub sync_vars: SyncVars,

    #[serde(skip_serializing, skip_deserializing)]
    pub index: DataIndex,
}

impl ContextData {
    pub fn read_quad_scene_shaders(&mut self) -> Result<(), Box<Error>> {
        for scene in self.quad_scenes.iter_mut() {
            scene.vert_src = file_to_string(&PathBuf::from(&scene.vert_src_path))?;
            scene.frag_src = file_to_string(&PathBuf::from(&scene.frag_src_path))?;
        }

        Ok(())
    }

    pub fn update_index(&mut self) {
        for (idx, scene) in self.quad_scenes.iter_mut().enumerate() {
            self.index.add_quad_scene(scene, idx);
        }

        for (idx, buffer) in self.frame_buffers.iter().enumerate() {
            self.index.add_frame_buffer(buffer, idx);
        }

    }
}

impl Default for ContextData {
    fn default() -> ContextData {
        ContextData {
            sync_vars: SyncVars::default(),
            quad_scenes: vec![],
            polygon_scenes: vec![],
            //polygon_context: PolygonContext::default(),
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

