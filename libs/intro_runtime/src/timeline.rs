use smallvec::SmallVec;

pub struct Timeline {
    pub tracks: SmallVec<[TimeTrack; 4]>,
}

impl Default for Timeline {
    fn default() -> Timeline {
        let mut track = TimeTrack {
            scene_blocks: SmallVec::new(),
        };

        let mut draw_ops: SmallVec<[DrawOp; 32]> = SmallVec::new();
        draw_ops.push(DrawOp::Target_Buffer_Default);
        // #4682B4, Steel Blue
        draw_ops.push(DrawOp::Clear(70, 130, 180, 0));

        let scene = SceneBlock {
            start: 0.0,
            end: 60.0,
            draw_ops: draw_ops,
        };

        track.scene_blocks.push(scene);

        let mut timeline = Timeline::new();
        timeline.tracks.push(track);

        timeline
    }
}

impl Timeline {
    pub fn new() -> Timeline {
        Timeline {
            tracks: SmallVec::new(),
        }
    }

    pub fn draw_ops_at_time(&self, time: f64) -> SmallVec<[DrawOp; 64]> {

        use self::DrawOp::*;

        // FIXME not very sophisticated at the moment, just copying the ops from
        // the first block, not even considering time.

        if self.tracks.len() > 0 {

            let track = &self.tracks[0];

            if track.scene_blocks.len() > 0 {

                let mut ops: SmallVec<[DrawOp; 64]> = SmallVec::new();

                for i in track.scene_blocks[0].draw_ops.iter() {
                    let o = match i {
                        NOOP => NOOP,
                        Exit(x) => Exit(*x),
                        Draw_Quad_Scene(x) => Draw_Quad_Scene(*x),
                        Draw_Poly_Scene(x) => Draw_Poly_Scene(*x),
                        Clear(r, g, b, a) => Clear(*r, *g, *b, *a),
                        Target_Buffer(x) => Target_Buffer(*x),
                        Target_Buffer_Default => Target_Buffer_Default,
                        Profile(x) => Profile(*x),
                    };
                    ops.push(o);
                }

                return ops;
            }
        }

        let mut ops: SmallVec<[DrawOp; 64]> = SmallVec::new();
        ops.push(DrawOp::Target_Buffer_Default);
        // #4682B4, Steel Blue
        ops.push(DrawOp::Clear(70, 130, 180, 0));

        return ops;
    }
}

pub struct TimeTrack {
    pub scene_blocks: SmallVec<[SceneBlock; 16]>,
}

pub struct SceneBlock {
    pub start: f64,
    pub end: f64,
    pub draw_ops: SmallVec<[DrawOp; 32]>,
}

pub enum DrawOp {
    NOOP,
    Exit(f64),
    Draw_Quad_Scene(usize),
    Draw_Poly_Scene(usize),
    Clear(u8, u8, u8, u8),
    Target_Buffer(usize),
    Target_Buffer_Default,
    Profile(usize),
}
