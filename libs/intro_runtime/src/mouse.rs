pub struct Mouse {
    /// Last cursor position X.
    pub last_x: i32,
    /// Last cursor position Y.
    pub last_y: i32,
    /// Movement X since last frame multiplied by `sensitivity * 1.5`.
    pub delta_x: f32,
    /// Movement Y since last frame multiplied by `sensitivity * 1.5`.
    pub delta_y: f32,
    /// Last click or current drag position X.
    pub last_click_drag_x: i32,
    /// Last click or current drag position Y.
    pub last_click_drag_y: i32,
    /// Drag start position X, negative if not dragging (i.e. last start drag position).
    pub drag_start_x: i32,
    /// Drag start position Y, negative if not dragging (i.e. last start drag position).
    pub drag_start_y: i32,
    /// Pressed buttons: [Left, Right, Middle].
    pub pressed: [bool; 3],
    /// Movement speed multiplier.
    pub sensitivity: f32,
}

impl Mouse {
    pub fn new(sensitivity: f32) -> Mouse {
        Mouse {
            last_x: 0,
            last_y: 0,
            delta_x: 0.0,
            delta_y: 0.0,
            last_click_drag_x: 0,
            last_click_drag_y: 0,
            drag_start_x: 0,
            drag_start_y: 0,
            pressed: [false, false, false],
            sensitivity,
        }
    }

    pub fn update_mouse_moved(&mut self, mouse_x: i32, mouse_y: i32) {
        // x1.5 because a bit stronger sensitivity is better on the delta
        self.delta_x = { mouse_x - self.last_x } as f32 * self.sensitivity * 1.5;
        self.delta_y = { mouse_y - self.last_y } as f32 * self.sensitivity * 1.5 * -1.0;
        self.last_x = mouse_x;
        self.last_y = mouse_y;
        if self.pressed[0] {
            self.last_click_drag_x = self.last_x;
            self.last_click_drag_y = self.last_y;
        }
    }

    pub fn update_mouse_input(&mut self, pressed: bool, button: MouseButton) {
        if pressed {
            self.last_click_drag_x = self.last_x;
            self.last_click_drag_y = self.last_y;
            if !self.pressed[0] {
                self.drag_start_x = self.last_x;
                self.drag_start_y = self.last_y;
            }
        } else {
            self.drag_start_x *= -1;
            self.drag_start_y *= -1;
        }

        match button {
            MouseButton::Left => self.pressed[0] = pressed,
            MouseButton::Right => self.pressed[1] = pressed,
            MouseButton::Middle => self.pressed[2] = pressed,
            _ => (),
        }
    }
}

pub enum MouseButton {
    NoButton,
    Left,
    Right,
    Middle,
}
