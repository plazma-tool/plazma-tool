pub struct Timeline {
    pub tracks: Vec<TimeTrack>,
}

impl Default for Timeline {
    fn default() -> Timeline {
        let mut track = TimeTrack {
            scene_blocks: Vec::new(),
        };

        let mut draw_ops: Vec<DrawOp> = Vec::new();
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
            tracks: Vec::new(),
        }
    }

    pub fn draw_ops_at_time(&self, time: f64) -> Vec<DrawOp> {
        use self::DrawOp::*;

        let mut ops: Vec<DrawOp> = Vec::new();

        // Always start by clearing the "RESULT_IMAGE" buffer.

        ops.push(DrawOp::Target_Buffer(0));
        ops.push(DrawOp::Clear(0, 0, 0, 0));

        if self.tracks.len() > 0 {
            for track in self.tracks.iter() {
                for block in track.scene_blocks.iter() {
                    if block.start <= time && block.end > time {
                        for i in block.draw_ops.iter() {
                            let o = match i {
                                NOOP => NOOP,
                                Draw_Quad_Scene(x) => Draw_Quad_Scene(*x),
                                Draw_Poly_Scene(x) => Draw_Poly_Scene(*x),
                                Clear(r, g, b, a) => Clear(*r, *g, *b, *a),
                                Target_Buffer(x) => Target_Buffer(*x),
                                Target_Buffer_Default => Target_Buffer_Default,
                                Profile(x) => Profile(*x),
                            };
                            ops.push(o);
                        }
                    }
                }
            }
        }

        // The user must always render the final result to the framebuffer named
        // "RESULT_IMAGE", which is implicitly created as the first item in the
        // array `context.framebuffers`.
        //
        // Draw ops have an implicit final sequence: select the default
        // framebuffer, clear with black, render a simple draw shader on a quad
        // (scene index 0), reading from the "RESULT_IMAGE" framebuffer.

        ops.push(DrawOp::Target_Buffer_Default);
        ops.push(DrawOp::Clear(0, 0, 0, 0));
        ops.push(DrawOp::Draw_Quad_Scene(0));

        return ops;
    }
}

pub struct TimeTrack {
    pub scene_blocks: Vec<SceneBlock>,
}

pub struct SceneBlock {
    pub start: f64,
    pub end: f64,
    pub draw_ops: Vec<DrawOp>,
}

pub enum DrawOp {
    NOOP,
    Draw_Quad_Scene(usize),
    Draw_Poly_Scene(usize),
    Clear(u8, u8, u8, u8),
    Target_Buffer(usize),
    Target_Buffer_Default,
    Profile(usize),
}
