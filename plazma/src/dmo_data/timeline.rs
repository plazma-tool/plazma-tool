#[derive(Serialize, Deserialize, Debug)]
pub struct Timeline {
    tracks: Vec<TimeTrack>,
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
                DrawOp::Clear(0, 0, 0, 0),
                DrawOp::Draw_Quad_Scene("default".to_string()),
            ],
        };

        track.scene_blocks.push(scene);

        Timeline {
            tracks: vec![
                track,
            ],
        }
    }
}

impl Timeline {
    pub fn draw_ops_at_time(&self, time: f64) -> Vec<DrawOp> {

        // FIXME not very sophisticated at the moment, just copying the ops from
        // the first block.

        if self.tracks.len() > 0 {

            let track = self.tracks.get(0).unwrap();

            if track.scene_blocks.len() > 0 {

                use self::DrawOp::*;
                let ops: Vec<DrawOp> = track
                    .scene_blocks.get(0).unwrap()
                    .draw_ops.iter()
                    .map(|i| {
                        match i {
                            NOOP => NOOP,
                            Exit(x) => Exit(*x),
                            Draw_Quad_Scene(x) => Draw_Quad_Scene(x.to_string()),
                            Draw_Poly_Scene(x) => Draw_Poly_Scene(x.to_string()),
                            Clear(r, g, b, a) => Clear(*r, *g, *b, *a),
                            Target_Buffer(x) => Target_Buffer(x.to_string()),
                            Target_Buffer_Default => Target_Buffer_Default,
                            Profile(x) => Profile(x.to_string()),
                        }
                    }).collect();

                return ops;
            }
        }

        let ops: Vec<DrawOp> = vec![
            DrawOp::Target_Buffer_Default,
            DrawOp::Clear(0, 255, 0, 0),
        ];

        return ops;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeTrack {
    scene_blocks: Vec<SceneBlock>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SceneBlock {
    start: f64,
    end: f64,
    draw_ops: Vec<DrawOp>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DrawOp {
    NOOP,
    Exit(f64),
    Draw_Quad_Scene(String),
    Draw_Poly_Scene(String),
    Clear(u8, u8, u8, u8),
    Target_Buffer(String),
    Target_Buffer_Default,
    Profile(String),
}

