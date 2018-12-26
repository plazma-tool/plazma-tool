use smallvec::SmallVec;
use rocket_sync::SyncDevice;
use dmo::Context;

use error::RuntimeError;
use error::RuntimeError::*;

pub struct DmoSync {
    pub device: SyncDevice,
    pub ops: SmallVec<[SyncOp; 64]>,
}

pub enum SyncOp {
    NOOP,
    Time_Var,
    Track_To_Var(u8, u8),
}

impl DmoSync {
    pub fn update_vars(&self, context: &mut Context) -> Result<(), RuntimeError> {
        use self::SyncOp::*;
        for op in self.ops.iter() {
            match *op {
                NOOP => (),

                Time_Var => context.set_time(self.device.time as f64 / 1000.0),

                Track_To_Var(track_idx, var_idx) => {
                    if track_idx as usize > self.device.tracks.len() - 1 {
                        return Err(TrackIdxIsOutOfBounds);
                    }
                    if var_idx as usize > context.vars.len() - 1 {
                        return Err(VarIdxIsOutOfBounds);
                    }

                    let x = self.device.tracks[track_idx as usize].value_at(self.device.row);
                    context.vars[var_idx as usize] = x;
                },
            }
        }
        Ok(())
    }
}

impl Default for DmoSync {
    fn default() -> DmoSync {
        DmoSync {
            device: SyncDevice::new(128.0, 8),
            ops: SmallVec::new(),
        }
    }
}
