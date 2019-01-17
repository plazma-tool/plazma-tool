use std::default::Default;

pub mod context_data;
pub mod quad_scene;
pub mod polygon_scene;
pub mod sync_vars;
pub mod timeline;

use crate::dmo_data::context_data::ContextData;
use crate::dmo_data::timeline::Timeline;

#[derive(Serialize, Deserialize, Debug)]
pub struct DmoData {
    /// User preferences and playback options.
    pub settings: Settings,

    /// Holds assets indexed by the Timeline and DrawOps, such as images, shader
    /// sources, sync tracks.
    pub context: ContextData,

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
            context: ContextData::default(),
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

