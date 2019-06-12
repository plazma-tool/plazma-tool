#[derive(Serialize, Deserialize, Debug)]
pub struct Timeline {
    pub tracks: Vec<TimeTrack>,
}

impl Default for Timeline {
    fn default() -> Timeline {
        let mut track = TimeTrack {
            scene_blocks: vec![],
        };

        let scene = SceneBlock {
            start: 0.0,
            end: 60.0,
            draw_ops: vec![
                DrawOp::Target_Buffer_Default,
                // #4682B4, Steel Blue
                DrawOp::Clear(70, 130, 180, 0),
            ],
        };

        track.scene_blocks.push(scene);

        Timeline {
            tracks: vec![track],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeTrack {
    pub scene_blocks: Vec<SceneBlock>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SceneBlock {
    pub start: f64,
    pub end: f64,
    pub draw_ops: Vec<DrawOp>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DrawOp {
    NOOP,
    Draw_Quad_Scene(String),
    Draw_Poly_Scene(String),
    Clear(u8, u8, u8, u8),
    Target_Buffer(String),
    Target_Buffer_Default,
    Profile(String),
}
