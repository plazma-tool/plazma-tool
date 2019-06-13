use smallvec::SmallVec;

use crate::error::RuntimeError::{self, *};
use crate::VAR_NUM;

/// `SyncVars` provide methods to query the `f64` array for reserverd and named
/// tracks such as `"time"`, or by index such as `235`.
pub struct SyncVars {
    pub tracks: SmallVec<[SyncTrack; VAR_NUM]>,
}

pub struct SyncTrack {
    pub name: [u8; 64],
    pub value: f64,
}

impl Default for SyncVars {
    fn default() -> SyncVars {
        // 31 tracks for 31 enum variants.
        SyncVars::new(31)
    }
}

impl SyncVars {
    pub fn new(tracks_count: usize) -> SyncVars {
        let mut tracks: SmallVec<[SyncTrack; VAR_NUM]> = SmallVec::new();

        // FIXME set the name of builtin tracks

        for _ in 0..=tracks_count {
            tracks.push(SyncTrack {
                name: [0; 64],
                value: 0.0,
            });
        }

        SyncVars { tracks: tracks }
    }

    pub fn add_tracks_up_to(&mut self, tracks_count: usize) {
        let n = tracks_count - self.tracks.len();
        if n <= 0 {
            return;
        }
        for _ in 0..=n {
            self.tracks.push(SyncTrack {
                name: [0; 64],
                value: 0.0,
            });
        }
    }

    pub fn get_index(&self, idx: usize) -> Result<f64, RuntimeError> {
        if self.tracks.len() > idx {
            Ok(self.tracks[idx].value)
        } else {
            Err(VarIdxIsOutOfBounds)
        }
    }

    pub fn set_index(&mut self, idx: usize, value: f64) -> Result<(), RuntimeError> {
        if self.tracks.len() > idx {
            self.tracks[idx].value = value;
            Ok(())
        } else {
            Err(VarIdxIsOutOfBounds)
        }
    }

    pub fn set_builtin(&mut self, name: BuiltIn, value: f64) {
        let idx = builtin_to_idx(name);
        self.tracks[idx].value = value;
    }

    pub fn get_builtin(&self, name: BuiltIn) -> f64 {
        let idx = builtin_to_idx(name);
        self.tracks[idx].value
    }

    /*
    pub fn set_custom(&mut self, name: &str, value: f64) {
        // TODO
    }

    pub fn get_custom(&self, name: &str) -> f64 {
        // TODO
        0.0
    }
    */
}

pub fn builtin_to_idx(name: BuiltIn) -> usize {
    use self::BuiltIn::*;
    match name {
        Time => 0,

        Window_Width => 1,
        Window_Height => 2,

        Screen_Width => 3,
        Screen_Height => 4,

        Camera_Pos_X => 5,
        Camera_Pos_Y => 6,
        Camera_Pos_Z => 7,

        Camera_Front_X => 8,
        Camera_Front_Y => 9,
        Camera_Front_Z => 10,

        Camera_Up_X => 11,
        Camera_Up_Y => 12,
        Camera_Up_Z => 13,

        Camera_LookAt_X => 14,
        Camera_LookAt_Y => 15,
        Camera_LookAt_Z => 16,

        Fovy => 17,
        Znear => 18,
        Zfar => 19,

        Light_Pos_X => 20,
        Light_Pos_Y => 21,
        Light_Pos_Z => 22,

        Light_Dir_X => 23,
        Light_Dir_Y => 24,
        Light_Dir_Z => 25,

        Light_Strength => 26,
        Light_Constant_Falloff => 27,
        Light_Linear_Falloff => 28,
        Light_Quadratic_Falloff => 29,
        Light_Cutoff_Angle => 30,

        // first n is 0
        Custom(n) => 31 + n,
    }
}

// NOTE remember to update SyncVars::default() when adding more enum variants.

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
