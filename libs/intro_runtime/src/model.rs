use smallvec::SmallVec;

use crate::mesh::Mesh;
use crate::context_gfx::ContextGfx;
use crate::types::{BufferMapping, UniformMapping};
use crate::error::RuntimeError;

pub struct Model {
    pub model_type: ModelType,
    // pub textures_loaded: SmallVec<[Texture; 2]>, // is this needed?
    pub meshes: SmallVec<[Mesh; 2]>,
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
            meshes: SmallVec::new(),
        }
    }

    pub fn empty_obj() -> Model {
        Model {
            model_type: ModelType::Obj,
            meshes: SmallVec::new(),
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
            meshes: SmallVec::new(),
        };

        // Add a mesh but no vertices, which will be created from already stored shapes.
        let mut mesh = Mesh::default();
        mesh.vert_src_idx = vert_src_idx;
        mesh.frag_src_idx = frag_src_idx;

        model.meshes.push(mesh);

        model
    }

    pub fn draw(&self,
                context: &ContextGfx,
                layout_to_vars: &SmallVec<[UniformMapping; 64]>,
                binding_to_buffers: &SmallVec<[BufferMapping; 64]>,
                model: &[[f32; 4]; 4],
                view: &[[f32; 4]; 4],
                projection: &[[f32; 4]; 4],
                camera_pos: &[f32; 3])
                -> Result<(), RuntimeError>
    {
        for m in self.meshes.iter() {
            m.draw(context, layout_to_vars, binding_to_buffers, model, view, projection, camera_pos)?;
        }
        Ok(())
    }

    pub fn gl_cleanup(&mut self) {
        for mut mesh in self.meshes.iter_mut() {
            mesh.gl_cleanup();
        }
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        self.gl_cleanup();
    }
}

