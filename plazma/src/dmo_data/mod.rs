use std::path::PathBuf;
use std::error::Error;

use tobj;

pub mod context_data;
pub mod data_index;
pub mod quad_scene;
pub mod polygon_scene;
pub mod polygon_context;
pub mod model;
pub mod sync_vars;
pub mod timeline;

use intro_runtime::dmo_gfx::DmoGfx;

use crate::dmo_data::context_data::{ContextData, FrameBuffer};
use crate::dmo_data::quad_scene::QuadScene;
use crate::dmo_data::timeline::Timeline;

#[derive(Serialize, Deserialize, Debug)]
pub struct DmoData {
    /// User preferences and playback options.
    pub settings: Settings,

    /// Holds assets indexed by the Timeline and DrawOps, such as images, shader
    /// sources, sync tracks.
    pub context: ContextData,

    /// Holds SceneBlocks on TimeTracks. Sampling the Timeline at time x
    /// produces a Vec<DrawOp> to draw the current frame by taking a
    /// cross-section of the Timeline and stacking the DrawOps on top of each
    /// other.
    pub timeline: Timeline,
}

impl Default for DmoData {
    fn default() -> DmoData {
        DmoData {
            settings: Settings::default(),
            context: ContextData::default(),
            timeline: Timeline::default(),
        }
    }
}

impl DmoData {
    pub fn new_from_yml_str(text: &str, read_shader_paths: bool) -> Result<DmoData, Box<Error>>
    {
        let mut dmo_data: DmoData = serde_yaml::from_str(text)?;
        dmo_data.ensure_implicit_builtins();
        dmo_data.context.build_index(read_shader_paths)?;
        Ok(dmo_data)
    }

    /// Ensures implicit builtins are included in the data. Skips them when
    /// already present. When the server is sending a serialized DmoData, the
    /// builtins will already be there.
    pub fn ensure_implicit_builtins(&mut self) {
        // Ensure "RESULT_IMAGE" framebuffer. Must have index 0, so we are going to prepend it.

        let mut has_result_image = false;
        for i in self.context.frame_buffers.iter() {
            if i.name == "RESULT_IMAGE" {
                has_result_image = true;
            }
        }

        if !has_result_image {
            let mut frame_buffers = vec![
                FrameBuffer::framebuffer_result_image()
            ];
            frame_buffers.append(&mut self.context.frame_buffers);
            self.context.frame_buffers = frame_buffers;

        }

        // Ensure "DRAW_RESULT" QuadScene. Must have index 0, so we are going to prepend it.

        let mut has_draw_result = false;
        for i in self.context.quad_scenes.iter() {
            if i.name == "DRAW_RESULT" {
                has_draw_result = true;
            }
        }

        if !has_draw_result {
            let mut quad_scenes = vec![
                QuadScene::scene_draw_result(),
            ];
            quad_scenes.append(&mut self.context.quad_scenes);
            self.context.quad_scenes = quad_scenes;
        }
    }

    pub fn add_models_to(&self, dmo_gfx: &mut DmoGfx) -> Result<(), Box<Error>> {
        use crate::dmo_data as d;

        for model_data in self.context.polygon_context.models.iter() {
            match model_data.model_type {
                d::model::ModelType::Cube => self.add_model_cube_to(dmo_gfx, model_data)?,
                d::model::ModelType::Obj => self.add_model_obj_to(dmo_gfx, model_data)?,
                d::model::ModelType::NOOP => {},
            }
        }

        Ok(())
    }

    pub fn add_model_cube_to(&self,
                            dmo_gfx: &mut DmoGfx,
                            model_data: &self::model::Model)
                            -> Result<(), Box<Error>>
    {
        use intro_runtime::model::Model;
        use intro_runtime::mesh::Mesh;

        let mut model = Model::empty_cube();

        let vert_src_idx = self.context.index.get_shader_index(&model_data.vert_src_path)?;
        let frag_src_idx = self.context.index.get_shader_index(&model_data.frag_src_path)?;

        // Add a mesh but no vertices, those will be created from shapes.
        let mut mesh = Mesh::default();
        mesh.vert_src_idx = vert_src_idx;
        mesh.frag_src_idx = frag_src_idx;

        model.meshes.push(mesh);

        dmo_gfx.context.polygon_context.models.push(model);

        Ok(())
    }

    pub fn add_model_obj_to(&self,
                        dmo_gfx: &mut DmoGfx,
                        model_data: &self::model::Model)
                        -> Result<(), Box<Error>>
    {
        use intro_runtime::model::Model;
        use intro_runtime::mesh::Mesh;
        use intro_runtime::types::Vertex;

        let mut model = Model::empty_obj();

        let vert_src_idx = self.context.index.get_shader_index(&model_data.vert_src_path)?;
        let frag_src_idx = self.context.index.get_shader_index(&model_data.frag_src_path)?;

        // if model_data.obj_path.len() == 0 {
        //     // that's a problem.
        //     // FIXME return an error
        // }

        // Add meshes.
        {
            let (meshes, _materials) = tobj::load_obj(&PathBuf::from(&model_data.obj_path))?;

            for i in meshes.iter() {
                let mesh = &i.mesh;
                let mut new_mesh = Mesh::default();

                // Serializing each vertex per index.
                // This will be a vertex array, not an indexed object using EBO.

                let n_index = mesh.indices.len();
                if n_index > std::u32::MAX as usize {
                    panic!{"Index list must not be over u32 max"};
                }

                // Add vertex data.
                for index in mesh.indices.iter() {
                    let i = *index as usize;

                    let position: [f32; 3] = [mesh.positions[3*i],
                                              mesh.positions[3*i+1],
                                              mesh.positions[3*i+2]];

                    let normal: [f32; 3] = [mesh.normals[3*i],
                                            mesh.normals[3*i+1],
                                            mesh.normals[3*i+2]];

                    let vertex = Vertex {
                        position: position,
                        normal: normal,
                        texcoords: [0.0; 2], // TODO UV texcoords
                    };

                    new_mesh.vertices.push(vertex);
                }

                // Use the shaders specified on the model for each mesh.
                new_mesh.vert_src_idx = vert_src_idx;
                new_mesh.frag_src_idx = frag_src_idx;

                model.meshes.push(new_mesh);
            }
        }

        dmo_gfx.context.polygon_context.models.push(model);

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub start_full_screen: bool,
    pub audio_play_on_start: bool,
    pub mouse_sensitivity: f32,
    pub movement_sensitivity: f32,
    pub total_length: f64,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            start_full_screen: false,
            audio_play_on_start: true,
            mouse_sensitivity: 0.5,
            movement_sensitivity: 0.5,
            total_length: 10.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ValueVec3 {
    NOOP,
    Sync(BuiltIn, BuiltIn, BuiltIn),
    Fixed(f32, f32, f32),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ValueFloat {
    NOOP,
    Sync(BuiltIn),
    Fixed(f32),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UniformMapping {
    NOOP,
    Float(u8, BuiltIn),
    Vec2(u8, BuiltIn, BuiltIn),
    Vec3(u8, BuiltIn, BuiltIn, BuiltIn),
    Vec4(u8, BuiltIn, BuiltIn, BuiltIn, BuiltIn),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BufferMapping {
    NOOP,
    Sampler2D(u8, String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BuiltIn {
    Time,

    Window_Width,
    Window_Height,

    Screen_Width,
    Screen_Height,

    Camera_Pos_X,
    Camera_Pos_Y,
    Camera_Pos_Z,

    Camera_Front_X,
    Camera_Front_Y,
    Camera_Front_Z,

    Camera_Up_X,
    Camera_Up_Y,
    Camera_Up_Z,

    Camera_LookAt_X,
    Camera_LookAt_Y,
    Camera_LookAt_Z,

    Fovy,
    Znear,
    Zfar,

    Light_Pos_X,
    Light_Pos_Y,
    Light_Pos_Z,

    Light_Dir_X,
    Light_Dir_Y,
    Light_Dir_Z,

    Light_Strength,
    Light_Constant_Falloff,
    Light_Linear_Falloff,
    Light_Quadratic_Falloff,
    Light_Cutoff_Angle,

    Custom(usize),
}

pub fn builtin_to_idx(name: &BuiltIn) -> usize {
    use self::BuiltIn::*;
    match name {
        Time                    => 0,

        Window_Width            => 1,
        Window_Height           => 2,

        Screen_Width            => 3,
        Screen_Height           => 4,

        Camera_Pos_X            => 5,
        Camera_Pos_Y            => 6,
        Camera_Pos_Z            => 7,

        Camera_Front_X          => 8,
        Camera_Front_Y          => 9,
        Camera_Front_Z          => 10,

        Camera_Up_X             => 11,
        Camera_Up_Y             => 12,
        Camera_Up_Z             => 13,

        Camera_LookAt_X         => 14,
        Camera_LookAt_Y         => 15,
        Camera_LookAt_Z         => 16,

        Fovy                    => 17,
        Znear                   => 18,
        Zfar                    => 19,

        Light_Pos_X             => 20,
        Light_Pos_Y             => 21,
        Light_Pos_Z             => 22,

        Light_Dir_X             => 23,
        Light_Dir_Y             => 24,
        Light_Dir_Z             => 25,

        Light_Strength          => 26,
        Light_Constant_Falloff  => 27,
        Light_Linear_Falloff    => 28,
        Light_Quadratic_Falloff => 29,
        Light_Cutoff_Angle      => 30,

        Custom(n)               => 31 + n,
    }
}
