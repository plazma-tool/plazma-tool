use intro_3d::lib::{Matrix4, Vector3};

use crate::error::RuntimeError;

use crate::context_gfx::ContextGfx;
use crate::types::{BufferMapping, UniformMapping, ValueFloat, ValueVec3};

pub struct PolygonScene {
    pub scene_objects: Vec<SceneObject>,
}

pub struct SceneObject {
    pub model_idx: usize,

    pub position: Vector3,
    pub euler_rotation: Vector3,
    pub scale: f32,

    pub position_var: ValueVec3,
    pub euler_rotation_var: ValueVec3,
    pub scale_var: ValueFloat,

    pub layout_to_vars: Vec<UniformMapping>,
    pub binding_to_buffers: Vec<BufferMapping>,

    /// Model matrix to use when drawing the model retreived with `model_idx`
    /// from `PolygonContext.models`.
    pub model_matrix: [[f32; 4]; 4],
}

impl Default for PolygonScene {
    fn default() -> PolygonScene {
        PolygonScene::empty()
    }
}

impl Default for SceneObject {
    fn default() -> SceneObject {
        SceneObject {
            model_idx: 0,

            position: Vector3::from_slice(&[0.0; 3]),
            euler_rotation: Vector3::from_slice(&[0.0; 3]),
            scale: 1.0,

            position_var: ValueVec3::Fixed(0.0, 0.0, 0.0),
            euler_rotation_var: ValueVec3::Fixed(0.0, 0.0, 0.0),
            scale_var: ValueFloat::Fixed(1.0),

            layout_to_vars: Vec::new(),
            binding_to_buffers: Vec::new(),

            // identity matrix
            model_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl PolygonScene {
    pub fn empty() -> PolygonScene {
        PolygonScene {
            scene_objects: Vec::new(),
        }
    }

    pub fn draw(&self, context: &ContextGfx) -> Result<(), RuntimeError> {
        for o in self.scene_objects.iter() {
            if let Some(ref model) = context.polygon_context.models.get(o.model_idx) {
                model.draw(
                    context,
                    &o.layout_to_vars,
                    &o.binding_to_buffers,
                    &o.model_matrix,
                    &context.polygon_context.view_matrix,
                    &context.polygon_context.projection_matrix,
                    &context.polygon_context.view_position.as_slice(),
                )?;
            }
        }
        Ok(())
    }
}

impl SceneObject {
    pub fn update_model_matrix(&mut self) {
        let a = Matrix4::new_homogeneous(&self.position, &self.euler_rotation, self.scale);
        self.model_matrix = a.as_column_slice();
    }
}
