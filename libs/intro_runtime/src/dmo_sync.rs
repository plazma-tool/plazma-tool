use rocket_sync::SyncDevice;

use crate::context_gfx::ContextGfx;
use crate::error::RuntimeError;
use crate::sync_vars::BuiltIn::*;

pub struct DmoSync {
    pub device: SyncDevice,
}

impl DmoSync {
    pub fn update_vars(&self, context: &mut ContextGfx) -> Result<(), RuntimeError> {

        // idx 0 is Time
        context.sync_vars.set_builtin(Time, self.device.time as f64 / 1000.0);

        // Get the Rocket track index for a given sync var idx and calculate the track's value.

        // FIXME this is assuming Rocket device track idx = Sync var idx
        //
        // FIXME starting with idx 5 because Time, Screen_Width, Screen_Height, Window_Width,
        // Window_Height shouldn't be set by Rocket tracks

        for idx in 5..self.device.tracks.len() {
            let x = self.device.tracks[idx].value_at(self.device.row);
            context.sync_vars.set_index(idx, x)?;
        }

        /*
        context.sync_vars.set_builtin(Camera_Pos_X, context.camera.position.x as f64);
        context.sync_vars.set_builtin(Camera_Pos_Y, context.camera.position.y as f64);
        context.sync_vars.set_builtin(Camera_Pos_Z, context.camera.position.z as f64);
        context.sync_vars.set_builtin(Camera_Front_X, context.camera.front.x as f64);
        context.sync_vars.set_builtin(Camera_Front_Y, context.camera.front.y as f64);
        context.sync_vars.set_builtin(Camera_Front_Z, context.camera.front.z as f64);
        context.sync_vars.set_builtin(Camera_Up_X, context.camera.up.x as f64);
        context.sync_vars.set_builtin(Camera_Up_Y, context.camera.up.y as f64);
        context.sync_vars.set_builtin(Camera_Up_Z, context.camera.up.z as f64);

        context.sync_vars.set_builtin(Fovy, context.camera.fovy_angle as f64);
        context.sync_vars.set_builtin(Znear, context.camera.clip_near as f64);
        context.sync_vars.set_builtin(Zfar, context.camera.clip_far as f64);
        */

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


