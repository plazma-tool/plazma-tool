use crate::dmo_data::quad_scene::QuadScene;
use crate::dmo_data::polygon_scene::PolygonScene;
use crate::dmo_data::sync_vars::SyncVars;

#[derive(Serialize, Deserialize, Debug)]
pub struct ContextData {
    pub sync_vars: SyncVars,
    pub quad_scenes: Vec<QuadScene>,
    pub polygon_scenes: Vec<PolygonScene>,
    //pub frame_buffers: Vec<FrameBuffer>,
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
            //frame_buffers: vec![],
            //poly_models: vec![],
            //audio_path: PathBuf::from(""),
        }
    }
}

