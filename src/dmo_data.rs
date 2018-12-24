use std::default::Default;
use std::path::PathBuf;

use serde_yaml;

use crate::utils::file_to_string;

#[derive(Serialize, Deserialize, Debug)]
pub struct DmoData {
    /// User preferences and playback options.
    pub settings: Settings,

    /// Holds assets indexed by the Timeline and DrawOps, such as images, shader
    /// sources, sync tracks.
    pub context_data: ContextData,

    /// Holds SceneBlocks on TimeTracks. Sampling the Timeline at time x
    /// produces a Vec<DrawOp> to draw the current frame by taking a
    /// cross-section of the Timeline and stacking the DrawOps on top of each
    /// other.
    pub timeline: Timeline,
}

impl Default for DmoData {
    fn default() -> DmoData {
        DmoData {
            settings: Settings::default(),
            context_data: ContextData::default(),
            timeline: Timeline::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub start_full_screen: bool,
    pub audio_play_on_start: bool,
    pub mouse_sensitivity: f32,
    pub movement_sensitivity: f32,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            start_full_screen: false,
            audio_play_on_start: true,
            mouse_sensitivity: 0.5,
            movement_sensitivity: 0.5,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ContextData {
    pub quad_scenes: Vec<QuadScene>,
    //pub poly_scenes: Vec<PolyScene>,
    //pub frame_buffers: Vec<FrameBuffer>,
    pub sync_vars: SyncVars,
    //pub audio_path: PathBuf,
}

impl Default for ContextData {
    fn default() -> ContextData {
        ContextData {
            quad_scenes: vec![
                QuadScene::default(),
            ],
            //poly_scenes: vec![],
            //frame_buffers: vec![],
            sync_vars: SyncVars::default(),
            //poly_models: vec![],
            //audio_path: PathBuf::from(""),
        }
    }
}

/// Not using a BTreeMap in preparation for `no_std`.
#[derive(Serialize, Deserialize, Debug)]
pub struct SyncVars {
    pub tracks: Vec<SyncTrack>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncTrack {
    pub name: String,
    pub value: f64,
}

impl Default for SyncVars {
    fn default() -> SyncVars {
        let text = include_str!("../data/default_sync_tracks.yml");
        let tracks: Vec<SyncTrack> = serde_yaml::from_str(&text).unwrap();

        SyncVars {
            tracks: tracks,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timeline {
    tracks: Vec<TimeTrack>,
 }

impl Default for Timeline {
    fn default() -> Timeline {
        let mut track = TimeTrack {
            scene_blocks: vec![],
        };

        let scene = SceneBlock {
            start: 0.0,
            end: 60.0,
            draw_ops: vec![
                DrawOp::Target_Buffer_Default,
                DrawOp::Clear(0, 0, 0, 0),
                DrawOp::Draw_Quad_Scene("default".to_string()),
            ],
        };

        track.scene_blocks.push(scene);

        Timeline {
            tracks: vec![
                track,
            ],
        }
    }
}

impl Timeline {
    pub fn draw_ops_at_time(&self, time: f64) -> Vec<DrawOp> {

        // FIXME not very sophisticated at the moment, just copying the ops from
        // the first block.

        if self.tracks.len() > 0 {

            let track = self.tracks.get(0).unwrap();

            if track.scene_blocks.len() > 0 {

                use self::DrawOp::*;
                let ops: Vec<DrawOp> = track
                    .scene_blocks.get(0).unwrap()
                    .draw_ops.iter()
                    .map(|i| {
                        match i {
                            NOOP => NOOP,
                            Exit(x) => Exit(*x),
                            Draw_Quad_Scene(x) => Draw_Quad_Scene(x.to_string()),
                            Draw_Poly_Scene(x) => Draw_Poly_Scene(x.to_string()),
                            Clear(r, g, b, a) => Clear(*r, *g, *b, *a),
                            Target_Buffer(x) => Target_Buffer(x.to_string()),
                            Target_Buffer_Default => Target_Buffer_Default,
                            Profile(x) => Profile(x.to_string()),
                        }
                    }).collect();

                return ops;
            }
        }

        let ops: Vec<DrawOp> = vec![
            DrawOp::Target_Buffer_Default,
            DrawOp::Clear(0, 255, 0, 0),
        ];

        return ops;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeTrack {
    scene_blocks: Vec<SceneBlock>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SceneBlock {
    start: f64,
    end: f64,
    draw_ops: Vec<DrawOp>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DrawOp {
    NOOP,
    Exit(f64),
    Draw_Quad_Scene(String),
    Draw_Poly_Scene(String),
    Clear(u8, u8, u8, u8),
    Target_Buffer(String),
    Target_Buffer_Default,
    Profile(String),
}

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
        let vert_src_path = include_str!("../data/screen_quad.vert");
        let frag_src_path = include_str!("../data/shader.frag");

        QuadScene {
            name: "default".to_string(),
            vert_src_path: vert_src_path.to_string(),
            vert_src: file_to_string(&PathBuf::from(vert_src_path)).expect("vert_src not found"),
            frag_src_path: frag_src_path.to_string(),
            frag_src: file_to_string(&PathBuf::from(frag_src_path)).expect("frag_src not found"),
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

/*
#[derive(Serialize, Deserialize, Debug)]
pub struct PolyScene {
    pub name: String,
    pub scene_objects: Vec<PolySceneObject>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PolySceneObject {
    pub model_name: String,
    pub position: ValueVec3,
    pub euler_rotation: ValueVec3,
    pub scale: ValueFloat,
    pub layout_to_vars: Vec<UniformMapping>,
    pub binding_to_buffers: Vec<BufferMapping>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ValueVec3 {
    NOOP,
    Sync(String, String, String),
    Fixed(f32, f32, f32),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ValueFloat {
    NOOP,
    Sync(String),
    Fixed(f32),
}
*/

#[derive(Serialize, Deserialize, Debug)]
pub enum UniformMapping {
    NOOP,
    Float(u8, String),
    Vec2(u8, String, String),
    Vec3(u8, String, String, String),
    Vec4(u8, String, String, String, String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BufferMapping {
    NOOP,
    Sampler2D(u8, String),
}

