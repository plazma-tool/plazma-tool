use smallvec::SmallVec;

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
        let mut tracks: SmallVec<[SyncTrack; VAR_NUM]> = SmallVec::new();
        // 0: time
        tracks.push(SyncTrack {name: [0; 64], value: 0.0});

        // 1, 2: window_resolution x y
        tracks.push(SyncTrack {name: [0; 64], value: 0.0});
        tracks.push(SyncTrack {name: [0; 64], value: 0.0});

        // 3, 4: screen_resolution x y
        tracks.push(SyncTrack {name: [0; 64], value: 0.0});
        tracks.push(SyncTrack {name: [0; 64], value: 0.0});

        SyncVars {
            tracks: tracks,
        }
    }
}

impl SyncVars {
    pub fn get_index(&self, idx: usize) -> f64 {
        if self.tracks.len() > idx {
            self.tracks[idx].value
        } else {
            // FIXME return error
            println!("can't get out of track idx bounds");
            0.0
        }
    }

    pub fn set_index(&mut self, idx: usize, value: f64) {
        if self.tracks.len() > idx {
            self.tracks[idx].value = value;
        } else {
            // FIXME return error
            println!("can't set out of track idx bounds");
        }
    }

    pub fn set_builtin(&mut self, name: BuiltIn, value: f64) {
        use self::BuiltIn::*;
        let idx = builtin_to_idx(name);
        self.tracks[idx].value = value;
    }

    pub fn get_builtin(&self, name: BuiltIn) -> f64 {
        let idx = builtin_to_idx(name);
        self.tracks[idx].value
    }

    pub fn set_custom(&mut self, name: &str, value: f64) {
        // TODO
    }

    pub fn get_custom(&self, name: &str) -> f64 {
        // TODO
        0.0
    }
}

pub fn builtin_to_idx(name: BuiltIn) -> usize {
    match name {
        Time          => 0,
        Window_Width  => 1,
        Window_Height => 2,
        Screen_Width  => 3,
        Screen_Height => 4,
    }
}

pub enum BuiltIn {
    Time,
    Window_Width,
    Window_Height,
    Screen_Width,
    Screen_Height,
}
