use crate::context_gfx::ContextGfx;
use crate::error::RuntimeError;
use crate::sync_vars::BuiltIn::*;

/// Stub for syncing with Rocket later on
pub struct DmoSync {
    pub device: SyncDevice,
}

/// Stub in place of rocket_sync::SyncDevice
pub struct SyncDevice {
    /// sync tracks (the vertical columns in the editor)
    //pub tracks: SmallVec<[SyncTrack; 64]>,
    /// rows per beat
    pub rpb: u8,
    /// beats per minute
    pub bpm: f64,
    /// rows per second
    pub rps: f64,
    pub is_paused: bool,
    /// current row
    pub row: u32,
    /// current time in milliseconds
    pub time: u32,
}

impl DmoSync {
    pub fn update_vars(&self, context: &mut ContextGfx) -> Result<(), RuntimeError> {
        context.sync_vars.set_builtin(Time, self.device.time as f64 / 1000.0);
        Ok(())
    }
}

impl Default for DmoSync {
    fn default() -> DmoSync {
        DmoSync {
            device: SyncDevice::new(128.0, 8),
        }
    }
}

impl SyncDevice {
    pub fn new(bpm: f64, rpb: u8) -> SyncDevice {
        SyncDevice {
            //tracks: SmallVec::new(),
            rpb: rpb,
            bpm: bpm,
            rps: rps(bpm, rpb),
            is_paused: true,
            row: 0,
            time: 0,
        }
    }

    pub fn set_row_from_time(&mut self) {
        let r: f64 = (self.time as f64 / 1000.0) * self.rps + 0.5;
        self.row = r as u32;
    }
}

/// Calculate rows per second
pub fn rps(bpm: f64, rpb: u8) -> f64 {
    (bpm / 60.0) * (rpb as f64)
}
