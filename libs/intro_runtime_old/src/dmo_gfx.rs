
pub struct Quad {
    pub program: GLuint,
    pub vao: GLuint,
    pub vbo: GLuint,
}

/// Order of attributes is significant, we want this to translate to a specific
/// data layout in memory and use byte offsets when setting the vertex
/// attributes.
#[derive(Clone)]
pub struct Vertex {
    pub position:  [GLfloat; 3],
    pub normal:    [GLfloat; 3],
    pub texcoords: [GLfloat; 2],
    //pub tangent:   [GLfloat; 3],
    //pub bitangent: [GLfloat; 3],
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

impl Drop for Quad {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}
