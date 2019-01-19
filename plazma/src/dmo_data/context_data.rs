use crate::dmo_data::quad_scene::QuadScene;
use crate::dmo_data::polygon_scene::PolygonScene;
//use crate::dmo_data::polygon_context::PolygonContext;
use crate::dmo_data::sync_vars::SyncVars;

#[derive(Serialize, Deserialize, Debug)]
pub struct ContextData {
    pub sync_vars: SyncVars,
    pub quad_scenes: Vec<QuadScene>,
    pub polygon_scenes: Vec<PolygonScene>,
    //pub polygon_context: PolygonContext,
    pub frame_buffers: Vec<FrameBuffer>,
    //pub audio_path: PathBuf,
}

impl Default for ContextData {
    fn default() -> ContextData {
        ContextData {
            sync_vars: SyncVars::default(),
            quad_scenes: vec![
                QuadScene::circle(),
                QuadScene::cross(),
            ],
            polygon_scenes: vec![],
            //polygon_context: PolygonContext::default(),
            frame_buffers: vec![
                FrameBuffer {
                    kind: BufferKind::Empty_Texture,
                    format: PixelFormat::RGBA_u8,
                },
            ],
            //poly_models: vec![],
            //audio_path: PathBuf::from(""),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FrameBuffer {
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

