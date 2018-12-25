use glium;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub tex: [f32; 2],
}

implement_vertex!(Vertex, pos, tex);

pub struct DmoGfx {
    pub quad_scenes: Vec<QuadSceneGfx>,
}

pub struct QuadSceneGfx {
    pub name: String,
    pub vbo: glium::VertexBuffer<Vertex>,
    pub indices: glium::index::NoIndices,
    pub program: glium::Program,
}
