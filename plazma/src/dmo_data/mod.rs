use std::path::PathBuf;
use std::error::Error;

//use serde_yaml;

pub mod context_data;
pub mod data_index;
pub mod quad_scene;
pub mod polygon_scene;
pub mod model;
pub mod mesh;
pub mod sync_vars;
pub mod timeline;

use crate::dmo_data::context_data::{ContextData, FrameBuffer};
use crate::dmo_data::quad_scene::QuadScene;
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

impl DmoData {
    pub fn new_from_yml_str(text: &str) -> Result<DmoData, Box<Error>> {
        let mut dmo_data: DmoData = serde_yaml::from_str(text)?;
        dmo_data.ensure_implicit_builtins();
        dmo_data.context.read_quad_scene_shaders()?;
        dmo_data.context.build_index();
        Ok(dmo_data)
    }

    pub fn ensure_implicit_builtins(&mut self) {
        // Ensure "RESULT_IMAGE" framebuffer. Index value is not significant.

        self.context.frame_buffers.push(FrameBuffer::framebuffer_result_image());

        // Ensure "DRAW_RESULT" QuadScene. Must have index 0.

        let mut quad_scenes = vec![
            QuadScene::scene_draw_result(),
        ];
        quad_scenes.append(&mut self.context.quad_scenes);

        self.context.quad_scenes = quad_scenes;
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
    Float(u8, BuiltIn),
    Vec2(u8, BuiltIn, BuiltIn),
    Vec3(u8, BuiltIn, BuiltIn, BuiltIn),
    Vec4(u8, BuiltIn, BuiltIn, BuiltIn, BuiltIn),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BufferMapping {
    NOOP,
    Sampler2D(u8, String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BuiltIn {
    Time,

    Window_Width,
    Window_Height,

    Screen_Width,
    Screen_Height,

    Camera_Pos_X,
    Camera_Pos_Y,
    Camera_Pos_Z,

    Camera_Front_X,
    Camera_Front_Y,
    Camera_Front_Z,

    Camera_Up_X,
    Camera_Up_Y,
    Camera_Up_Z,

    Camera_LookAt_X,
    Camera_LookAt_Y,
    Camera_LookAt_Z,

    Fovy,
    Znear,
    Zfar,

    Light_Pos_X,
    Light_Pos_Y,
    Light_Pos_Z,

    Light_Dir_X,
    Light_Dir_Y,
    Light_Dir_Z,

    Light_Strength,
    Light_Constant_Falloff,
    Light_Linear_Falloff,
    Light_Quadratic_Falloff,
    Light_Cutoff_Angle,

    Custom(usize),
}

pub fn builtin_to_idx(name: &BuiltIn) -> usize {
    use self::BuiltIn::*;
    match name {
        Time                    => 0,

        Window_Width            => 1,
        Window_Height           => 2,

        Screen_Width            => 3,
        Screen_Height           => 4,

        Camera_Pos_X            => 5,
        Camera_Pos_Y            => 6,
        Camera_Pos_Z            => 7,

        Camera_Front_X          => 8,
        Camera_Front_Y          => 9,
        Camera_Front_Z          => 10,

        Camera_Up_X             => 11,
        Camera_Up_Y             => 12,
        Camera_Up_Z             => 13,

        Camera_LookAt_X         => 14,
        Camera_LookAt_Y         => 15,
        Camera_LookAt_Z         => 16,

        Fovy                    => 17,
        Znear                   => 18,
        Zfar                    => 19,

        Light_Pos_X             => 20,
        Light_Pos_Y             => 21,
        Light_Pos_Z             => 22,

        Light_Dir_X             => 23,
        Light_Dir_Y             => 24,
        Light_Dir_Z             => 25,

        Light_Strength          => 26,
        Light_Constant_Falloff  => 27,
        Light_Linear_Falloff    => 28,
        Light_Quadratic_Falloff => 29,
        Light_Cutoff_Angle      => 30,

        Custom(n)               => 31 + n,
    }
}
