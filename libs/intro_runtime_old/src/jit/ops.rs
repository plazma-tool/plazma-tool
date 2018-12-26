use dmo::Context;

pub trait Ops {
    extern "sysv64" fn op_exit(&mut self, limit: f64);

    extern "sysv64" fn op_draw_quad_scene(&self, scene_id: u8);
    extern "sysv64" fn op_if_var_equal_draw_quad(&self, var_idx: u8, value: f64, scene_idx: u8);
    extern "sysv64" fn op_if_var_equal_draw_polygon(&self, var_idx: u8, value: f64, scene_idx: u8);
    extern "sysv64" fn op_clear(&self, red: u8, green: u8, blue: u8, alpha: u8);

    extern "sysv64" fn op_target_buffer(&self, buffer_idx: u8);
    extern "sysv64" fn op_target_buffer_default(&self);

    extern "sysv64" fn op_profile_event(&mut self, label_idx: u8);
}

impl Ops for Context {

    /// `.is_running` is the break condition for the main drawing loop.
    /// This sets `.is_running` to `false` if `.time` is over the `limit`.
    extern "sysv64" fn op_exit(&mut self, limit: f64) {
        self.impl_exit(limit);
    }

    extern "sysv64" fn op_draw_quad_scene(&self, scene_idx: u8) {
        self.impl_draw_quad_scene(scene_idx as usize);
    }

    extern "sysv64" fn op_if_var_equal_draw_quad(&self, var_idx: u8, value: f64, scene_idx: u8) {
        self.impl_if_var_equal_draw_quad(var_idx as usize, value, scene_idx as usize);
    }

    extern "sysv64" fn op_if_var_equal_draw_polygon(&self, var_idx: u8, value: f64, scene_idx: u8) {
        self.impl_if_var_equal_draw_polygon(var_idx as usize, value, scene_idx as usize);
    }

    extern "sysv64" fn op_clear(&self, red: u8, green: u8, blue: u8, alpha: u8) {
        self.impl_clear(red, green, blue, alpha);
    }

    extern "sysv64" fn op_target_buffer(&self, buffer_idx: u8) {
        self.impl_target_buffer(buffer_idx as usize);
    }

    extern "sysv64" fn op_target_buffer_default(&self) {
        self.impl_target_buffer_default();
    }

    extern "sysv64" fn op_profile_event(&mut self, label_idx: u8) {
        self.impl_profile_event(label_idx as usize);
    }
}
