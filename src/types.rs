use std::sync::{Arc, Mutex};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Gui {
    pub time: f32,
}

impl Default for Gui {
    fn default() -> Gui {
        Gui {
            time: 0.0,
        }
    }
}

pub struct AppState {
    pub gui: Gui,
}

pub type AppStateWrap = Arc<Mutex<AppState>>;

impl AppState {
    pub fn new() -> AppState {
        AppState {
            gui: Gui::default(),
        }
    }
}

