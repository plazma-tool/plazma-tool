use std::error::Error;
use std::time::{Duration, Instant};
use std::path::PathBuf;

use glutin::VirtualKeyCode;

use smallvec::SmallVec;

use tobj;
use intro_3d::Vector3;
use intro_runtime::ERR_MSG_LEN;
use intro_runtime::dmo_gfx::DmoGfx;
use intro_runtime::dmo_sync::SyncDevice;
use intro_runtime::sync_vars::builtin_to_idx;
use intro_runtime::sync_vars::BuiltIn::*;
use intro_runtime::frame_buffer::{FrameBuffer, BufferKind};
use intro_runtime::mesh::Mesh;
use intro_runtime::polygon_scene::{PolygonScene, SceneObject};
use intro_runtime::model::{Model, ModelType};
use intro_runtime::types::{PixelFormat, Vertex, ValueVec3, ValueFloat};
use intro_runtime::error::RuntimeError;
use intro_runtime::types::{BufferMapping, UniformMapping};

use crate::dmo_data::DmoData;
use crate::error::ToolError;
use crate::utils::file_to_string;

pub struct PreviewState {
    pub t_frame_start: Instant,
    pub t_delta: Duration,
    pub t_frame_target: Duration,

    pub pressed_keys: [bool; 1024],
    pub explore_mode: bool,

    pub draw_anyway: bool,
    pub should_recompile: bool,
    pub movement_speed: f32,

    pub dmo_gfx: DmoGfx,
}

impl PreviewState {

    pub fn new(window_width: f64, window_height: f64,
               screen_width: f64, screen_height: f64)
               -> Result<PreviewState, Box<Error>>
    {
        let mut state = PreviewState {
            t_frame_start: Instant::now(),
            t_delta: Duration::new(0, 0),
            t_frame_target: Duration::from_millis(16),

            pressed_keys: [false; 1024],
            explore_mode: false,

            draw_anyway: false,
            should_recompile: false,
            movement_speed: 0.5,

            dmo_gfx: DmoGfx::default(),
        };

        state.set_window_resolution(window_width, window_height);
        state.set_screen_resolution(screen_width, screen_height);

        Ok(state)
    }

    pub fn build(&mut self, dmo_data: &DmoData) -> Result<(), Box<Error>> {

        // Manually for now. First add objects, then create OpenGL objects.

        // Add frame buffers

        self.dmo_gfx.context.frame_buffers.push(
            FrameBuffer::new(BufferKind::Empty_Texture,
                             PixelFormat::RGBA_u8,
                             None)
        );

        // Add quad scenes

        for quad_scene_data in dmo_data.context.quad_scenes.iter() {
            if quad_scene_data.frag_src_path.ends_with("circle.frag") {
                let mut layout_to_vars: SmallVec<[UniformMapping; 64]> = SmallVec::new();

                layout_to_vars.push(UniformMapping::Float(0,
                                                          builtin_to_idx(Time) as u8));

                layout_to_vars.push(UniformMapping::Vec2(1,
                                                         builtin_to_idx(Window_Width) as u8,
                                                         builtin_to_idx(Window_Height) as u8));

                layout_to_vars.push(UniformMapping::Vec2(2,
                                                         builtin_to_idx(Screen_Width) as u8,
                                                         builtin_to_idx(Screen_Height) as u8));

                self.dmo_gfx
                    .context
                    .add_quad_scene(&quad_scene_data.vert_src,
                                    &quad_scene_data.frag_src,
                                    layout_to_vars,
                                    SmallVec::new());
            }
            else if quad_scene_data.frag_src_path.ends_with("cross.frag") {
                let mut layout_to_vars: SmallVec<[UniformMapping; 64]> = SmallVec::new();

                layout_to_vars.push(UniformMapping::Float(0,
                                                          builtin_to_idx(Time) as u8));

                layout_to_vars.push(UniformMapping::Vec2(1,
                                                         builtin_to_idx(Window_Width) as u8,
                                                         builtin_to_idx(Window_Height) as u8));

                layout_to_vars.push(UniformMapping::Vec2(2,
                                                         builtin_to_idx(Screen_Width) as u8,
                                                         builtin_to_idx(Screen_Height) as u8));

                let mut binding_to_buffers: SmallVec<[BufferMapping; 64]> = SmallVec::new();

                binding_to_buffers.push(BufferMapping::Sampler2D(0, 0));

                self.dmo_gfx
                    .context
                    .add_quad_scene(&quad_scene_data.vert_src,
                                    &quad_scene_data.frag_src,
                                    layout_to_vars,
                                    binding_to_buffers);
            }
        }

        // Create quads

        let mut err_msg_buf = [32 as u8; ERR_MSG_LEN];

        match self.dmo_gfx.create_quads(&mut err_msg_buf) {
            Ok(_) => {},
            Err(e) => {
                let msg = String::from_utf8(err_msg_buf.to_vec())?;
                return Err(Box::new(ToolError::Runtime(e, msg)));
            }
        }

        // Create framebuffers

        match self.dmo_gfx.create_frame_buffers() {
            Ok(_) => {},
            Err(e) => return Err(Box::new(ToolError::Runtime(e, "".to_owned())))
        }

        // === Polygon scene: Cube, Suzanne, Skull, Dragon ===

        // read in object shaders and store indexes
        let scene_object_vert_src_idx =
            self.add_shader_src(&PathBuf::from("./data/shaders/scene_object.vert"));
        let cube_frag_src_idx =
            self.add_shader_src(&PathBuf::from("./data/shaders/cube_one.frag"));
        let suzanne_frag_src_idx =
            self.add_shader_src(&PathBuf::from("./data/shaders/suzanne.frag"));
        let skull_frag_src_idx =
            self.add_shader_src(&PathBuf::from("./data/shaders/skull.frag"));
        let dragon_vert_src_idx =
            self.add_shader_src(&PathBuf::from("./data/shaders/dragon.vert"));
        let dragon_frag_src_idx =
            self.add_shader_src(&PathBuf::from("./data/shaders/dragon.frag"));

        // Set PolygonContext sync vars.

        self.dmo_gfx.context.sync_vars.set_builtin(Camera_Pos_X, -5.0);
        self.dmo_gfx.context.sync_vars.set_builtin(Camera_Pos_Y, 4.0);
        self.dmo_gfx.context.sync_vars.set_builtin(Camera_Pos_Z, 10.0);

        self.dmo_gfx.context.sync_vars.set_builtin(Camera_Front_X, 0.5);
        self.dmo_gfx.context.sync_vars.set_builtin(Camera_Front_Y, -0.1);
        self.dmo_gfx.context.sync_vars.set_builtin(Camera_Front_Z, -0.5);

        // Add model: 0 (Cube)
        {
            let mut model = Model::default();
            model.model_type = ModelType::Cube;

            // Add a mesh but no vertices, those will be created from shapes.
            {
                let mut mesh = Mesh::default();

                mesh.vert_src_idx = scene_object_vert_src_idx;
                mesh.frag_src_idx = cube_frag_src_idx;

                model.meshes.push(mesh);
            }

            self.dmo_gfx.context.polygon_context.models.push(model);
        }

        // Add model: 1 (Suzanne)
        self.add_model_from_obj(&PathBuf::from("./data/obj/suzanne.obj"),
                                scene_object_vert_src_idx,
                                suzanne_frag_src_idx)?;

        // Add model: 2 (Skull)
        self.add_model_from_obj(&PathBuf::from("./data/obj/skull.obj"),
                                scene_object_vert_src_idx,
                                skull_frag_src_idx)?;

        // Add model: 3 (Dragon)
        self.add_model_from_obj(&PathBuf::from("./data/obj/dragon.obj"),
                                dragon_vert_src_idx,
                                dragon_frag_src_idx)?;

        // Create the OpenGL objects

        match self.dmo_gfx.create_models(&mut err_msg_buf) {
            Ok(_) => {},
            Err(e) => {
                let msg = String::from_utf8(err_msg_buf.to_vec())?;
                return Err(Box::new(ToolError::Runtime(e, msg)));
            }
        }

        let mut polygon_scene = PolygonScene::default();

        // Add SceneObjects.

        // Four objects, 0 1 2 3, corresponding to model indices.
        for i in 0..4 {
            let mut scene_object = SceneObject::default();

            scene_object.model_idx = i;

            scene_object.position_var = ValueVec3::Fixed(4.0, 0.0, 2.0 - (i as f32)*1.5);
            scene_object.scale_var = ValueFloat::Fixed(1.0);

            // When drawing a polygon mesh, uniform locations 0, 1, 2, 3 are
            // always bound to model, view, projection and view_pos.
            //
            // Further locations are bound with layout_to_vars.

            scene_object.layout_to_vars.push(
                UniformMapping::Float(4,
                                      builtin_to_idx(Time) as u8));

            scene_object.layout_to_vars.push(
                UniformMapping::Vec2(5,
                                     builtin_to_idx(Window_Width) as u8,
                                     builtin_to_idx(Window_Height) as u8));

            scene_object.layout_to_vars.push(
                UniformMapping::Vec2(6,
                                     builtin_to_idx(Screen_Width) as u8,
                                     builtin_to_idx(Screen_Height) as u8));

            scene_object.layout_to_vars.push(
                UniformMapping::Vec3(7,
                                     builtin_to_idx(Light_Pos_X) as u8,
                                     builtin_to_idx(Light_Pos_Y) as u8,
                                     builtin_to_idx(Light_Pos_Z) as u8));

            match self.dmo_gfx.compile_model_shaders(scene_object.model_idx, &mut err_msg_buf) {
                Ok(_) => {},
                Err(e) => {
                    let msg = String::from_utf8(err_msg_buf.to_vec())?;
                    return Err(Box::new(ToolError::Runtime(e, msg)));
                }
            }

            polygon_scene.scene_objects.push(scene_object);

        }

        self.dmo_gfx.context.polygon_scenes.push(polygon_scene);

        let aspect = self.dmo_gfx.context.get_window_aspect();
        self.dmo_gfx.context.polygon_context.update_projection_matrix(aspect as f32);


        Ok(())
    }

    fn add_shader_src(&mut self, path: &PathBuf) -> usize {
        let src = file_to_string(path).unwrap();
        self.dmo_gfx.context.shader_sources.push(SmallVec::from_slice(src.as_bytes()));
        return self.dmo_gfx.context.shader_sources.len() - 1;
    }

    fn add_model_from_obj(&mut self,
                          path: &PathBuf,
                          vert_src_idx: usize,
                          frag_src_idx: usize)
                          -> Result<(), Box<Error>>
    {
        let mut model = Model::default();
        model.model_type = ModelType::Obj;

        // Add meshes.
        {
            let (meshes, _materials) = tobj::load_obj(&path)?;

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

        self.dmo_gfx.context.polygon_context.models.push(model);

        Ok(())
    }

    pub fn recompile_dmo(&mut self) {

        // Compile a single QuadScene for now.

        if self.should_recompile {
            let mut err_msg_buf = [32 as u8; ERR_MSG_LEN];
            match self.dmo_gfx.compile_quad_scene(0, &mut err_msg_buf) {
                Ok(_) => {
                    println!("Recompiled");
                },
                Err(e) => {
                    // FIXME return error
                    let msg = String::from_utf8(err_msg_buf.to_vec()).unwrap();
                    println!("Recompile error:\n{}", msg);
                    //return Err(Box::new(ToolError::Runtime(e, msg)));
                }
            }

            self.should_recompile = false;
        }

    }

    pub fn draw(&self) {
        self.dmo_gfx.draw();
    }

    pub fn update_time_frame_start(&mut self) {
        self.t_frame_start = Instant::now();
        self.dmo_gfx.update_time_frame_start(self.t_frame_start);

        if !self.get_is_paused() {
            let d = self.get_sync_device_mut();
            d.time += 16;// 1s / 60 frames
            d.set_row_from_time();
        }
    }

    pub fn update_time_frame_end(&mut self) {
        self.dmo_gfx.update_time_frame_end(Instant::now());
    }

    pub fn update_vars(&mut self) -> Result<(), RuntimeError> {
        // When playing, sync values from Rocket and
        // update the widget values to synced values.
        if !self.get_is_paused() {
            self.dmo_gfx.update_vars()?;
        }

        Ok(())
    }

    pub fn callback_window_resized(&mut self, wx: f64, wy: f64) -> Result<(), Box<Error>> {
        self.dmo_gfx.context.set_window_resolution(wx, wy);
        match self.dmo_gfx.recreate_framebuffers() {
            Ok(_) => {},
            Err(e) => return Err(From::from(format!("{:?}", e))),
        };
        self.draw_anyway = true;

        let window_aspect = wx as f32 / wy as f32;
        // self.camera.aspect = window_aspect;
        // self.camera.update_projection();

        Ok(())
    }

    // -- get and set --

    pub fn get_is_running(&self) -> bool {
        self.dmo_gfx.context.is_running
    }

    pub fn set_is_running(&mut self, value: bool) {
        self.dmo_gfx.context.is_running = value;
    }

    pub fn get_is_paused(&self) -> bool {
        self.dmo_gfx.sync.device.is_paused
    }

    pub fn get_sync_time(&self) -> u32 {
        self.dmo_gfx.sync.device.time
    }

    pub fn set_sync_time(&mut self, time: u32) {
        self.dmo_gfx.sync.device.time = time;
    }

    pub fn set_sync_row_from_time(&mut self) {
        self.dmo_gfx.sync.device.set_row_from_time();
    }

    pub fn set_is_paused(&mut self, value: bool) {
        self.dmo_gfx.sync.device.is_paused = value;
    }

    pub fn get_sync_device(&self) -> &SyncDevice {
        &self.dmo_gfx.sync.device
    }

    pub fn get_sync_device_mut(&mut self) -> &mut SyncDevice {
        &mut self.dmo_gfx.sync.device
    }

    pub fn get_window_resolution(&self) -> (f64, f64) {
        self.dmo_gfx.context.get_window_resolution()
    }

    pub fn set_window_resolution(&mut self, wx: f64, wy: f64) {
        self.dmo_gfx.context.set_window_resolution(wx, wy)
    }

    pub fn get_screen_resolution(&self) -> (f64, f64) {
        self.dmo_gfx.context.get_screen_resolution()
    }

    pub fn set_screen_resolution(&mut self, wx: f64, wy: f64) {
        self.dmo_gfx.context.set_screen_resolution(wx, wy)
    }

    pub fn get_t_delta_as_nanos(&self) -> u64 {
        (self.t_delta.as_secs() * 1_000_000_000) + (self.t_delta.subsec_nanos() as u64)
    }

    pub fn get_t_frame_target_as_nanos(&self) -> u64 {
        (self.t_frame_target.as_secs() * 1_000_000_000) + (self.t_frame_target.subsec_nanos() as u64)
    }

    pub fn set_key_pressed(&mut self, vcode: VirtualKeyCode, pressed: bool) {
        let n = vcode as usize;
        if n < self.pressed_keys.len() {
            self.pressed_keys[n] = pressed;
        }
    }

    pub fn set_camera_from_context(&mut self) {
        // FIXME set pitch and yaw from the front vector. This is what causes
        // the angle jump when explore mode takes over camera control.

        let a = self.get_context_camera_position();
        self.dmo_gfx.context.camera.set_position(a);
        let a = self.get_context_camera_front();
        self.dmo_gfx.context.camera.set_front(a);
    }

    pub fn get_context_camera_position(&self) -> Vector3 {
        let position: Vector3 = Vector3::new(
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Pos_X) as f32,
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Pos_Y) as f32,
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Pos_Z) as f32);
        position
    }

    pub fn get_context_camera_front(&self) -> Vector3 {
        let front: Vector3 = Vector3::new(
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Front_X) as f32,
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Front_Y) as f32,
            self.dmo_gfx.context.sync_vars.get_builtin(Camera_Front_Z) as f32);
        front
    }

    pub fn update_camera_from_keys(&mut self) {
        if self.pressed_keys[VirtualKeyCode::W as usize] {
            self.dmo_gfx.context.camera.move_forward(self.movement_speed);
            self.draw_anyway = true;
        }
        if self.pressed_keys[VirtualKeyCode::S as usize] {
            self.dmo_gfx.context.camera.move_backward(self.movement_speed);
            self.draw_anyway = true;
        }
        if self.pressed_keys[VirtualKeyCode::A as usize] {
            self.dmo_gfx.context.camera.move_left(self.movement_speed);
            self.draw_anyway = true;
        }
        if self.pressed_keys[VirtualKeyCode::D as usize] {
            self.dmo_gfx.context.camera.move_right(self.movement_speed);
            self.draw_anyway = true;
        }
        self.dmo_gfx.context.camera.update_view();
    }
}
