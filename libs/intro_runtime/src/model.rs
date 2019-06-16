use crate::context_gfx::ContextGfx;
use crate::error::RuntimeError;
use crate::mesh::Mesh;
use crate::types::{BufferMapping, UniformMapping};

pub struct Model {
    pub model_type: ModelType,
    // pub textures_loaded: Vec<Texture>, // is this needed?
    pub meshes: Vec<Mesh>,
}

pub struct ModelViewProjection {
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
}

#[derive(Copy, Clone)]
pub enum ModelType {
    NOOP,
    Cube,
    Obj,
}

impl Default for Model {
    fn default() -> Model {
        Model::empty_cube()
    }
}

impl Model {
    pub fn empty_cube() -> Model {
        Model {
            model_type: ModelType::Cube,
            meshes: Vec::new(),
        }
    }

    pub fn empty_obj() -> Model {
        Model {
            model_type: ModelType::Obj,
            meshes: Vec::new(),
        }
    }

    pub fn new(model_type: ModelType) -> Model {
        let mut m = Model::default();
        m.model_type = model_type;
        m
    }

    pub fn cube(vert_src_idx: usize, frag_src_idx: usize) -> Model {
        let mut model = Model {
            model_type: ModelType::Cube,
            meshes: Vec::new(),
        };

        // Add a mesh but no vertices, which will be created from already stored shapes.
        let mut mesh = Mesh::default();
        mesh.vert_src_idx = vert_src_idx;
        mesh.frag_src_idx = frag_src_idx;

        model.meshes.push(mesh);

        model
    }

    pub fn draw(
        &self,
        context: &ContextGfx,
        layout_to_vars: &[UniformMapping],
        binding_to_buffers: &[BufferMapping],
        mvp: &ModelViewProjection,
        camera_pos: &[f32; 3],
    ) -> Result<(), RuntimeError> {
        for m in self.meshes.iter() {
            m.draw(context, layout_to_vars, binding_to_buffers, mvp, camera_pos)?;
        }
        Ok(())
    }

    pub fn gl_cleanup(&mut self) {
        for mesh in self.meshes.iter_mut() {
            mesh.gl_cleanup();
        }
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}
