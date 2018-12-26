use core::{mem, ptr, str};

use smallvec::SmallVec;
use rocket_sync::{SyncDevice, SyncTrack, TrackKey, code_to_key};

use dmo::{Dmo, Context, Image, PixelFormat, Operator, QuadScene, Vertex,
PolygonScene, SceneObject, PolygonContext, Model, ModelType, Mesh, FrameBuffer,
BufferKind, UniformMapping, BufferMapping, ValueVec3, ValueFloat};

use sync::{DmoSync, SyncOp};

pub fn dmo_from_bytecode(data: SmallVec<[u8; 0x8000]>,
                         window_width: u32,
                         window_height: u32,
                         screen_width: u32,
                         screen_height: u32)
                         -> Dmo {
    let mut blob = DataBlob::new(data);

    // === Context ===

    // Shader sources

    let mut n_shaders = blob.read_u8();
    let mut shader_sources: SmallVec<[SmallVec<[u8; 1024]>; 64]> = SmallVec::new();

    while n_shaders >= 1 {
        let l = blob.read_u32();
        shader_sources.push(blob.read_u8_vec(l as usize));
        n_shaders -= 1;
    }

    // Images

    let mut n_images = blob.read_u8();
    let mut images: SmallVec<[Image; 4]> = SmallVec::new();

    while n_images >= 1 {
        let width = blob.read_u32();
        let height = blob.read_u32();
        let format = code_to_format(blob.read_u8());
        let bytes_per_pixel = match format {
            PixelFormat::NOOP => 0,
            PixelFormat::RED_u8 => 1,
            PixelFormat::RGB_u8 => 3,
            PixelFormat::RGBA_u8 => 4,
        };

        let n_bytes: usize = (width as usize) * (height as usize) * bytes_per_pixel;

        images.push(Image{
            width: width,
            height: height,
            format: format,
            raw_pixels: blob.read_u8_vec(n_bytes),
        });

        n_images -= 1;
    }

    // Framebuffers

    let mut n_frame_buffers = blob.read_u8();
    let mut frame_buffers: SmallVec<[FrameBuffer; 64]> = SmallVec::new();

    while n_frame_buffers >= 1 {
        let kind = code_to_bufkind(blob.read_u8());
        let mut has_image = false;
        let mut is_noop = false;

        match kind {
            BufferKind::NOOP => { is_noop = true; },
            BufferKind::Empty_Texture => {},
            BufferKind::Image_Texture => { has_image = true; },
        }

        if !is_noop {
            let format = code_to_format(blob.read_u8());
            if has_image {
                let image_data_idx = blob.read_u8();
                frame_buffers.push(FrameBuffer::new(kind, format, Some(image_data_idx as usize)))
            } else {
                frame_buffers.push(FrameBuffer::new(kind, format, None))
            }
        }

        n_frame_buffers -= 1;
    }

    // Quad Scenes

    let mut n_quad_scenes = blob.read_u8();
    let mut quad_scenes: SmallVec<[QuadScene; 64]> = SmallVec::new();

    while n_quad_scenes >= 1 {
        let mut scene: QuadScene = QuadScene{
            id: 0,
            vert_src_idx: 0,
            frag_src_idx: 0,
            layout_to_vars: SmallVec::new(),
            binding_to_buffers: SmallVec::new(),
            quad: None,
        };

        scene.vert_src_idx = blob.read_u8() as usize;
        scene.frag_src_idx = blob.read_u8() as usize;

        let mut n_mappings = blob.read_u8();

        while n_mappings >= 1 {
            let uni = code_to_uni(blob.read_u8());

            use dmo::UniformMapping::*;
            let unival = match uni {
                NOOP => NOOP,

                Float(_, _) => Float(blob.read_u8(),
                                     blob.read_u8()),

                Vec2(_, _, _) => Vec2(blob.read_u8(),
                                      blob.read_u8(),
                                      blob.read_u8()),

                Vec3(_, _, _, _) => Vec3(blob.read_u8(),
                                         blob.read_u8(),
                                         blob.read_u8(),
                                         blob.read_u8()),

                Vec4(_, _, _, _, _) => Vec4(blob.read_u8(),
                                            blob.read_u8(),
                                            blob.read_u8(),
                                            blob.read_u8(),
                                            blob.read_u8()),

            };

            match unival {
                NOOP => {},
                _ => scene.layout_to_vars.push(unival),
            }

            n_mappings -= 1;
        }

        let mut n_buf_mappings = blob.read_u8();

        while n_buf_mappings >= 1 {
            let buf = code_to_buf(blob.read_u8());

            use dmo::BufferMapping::*;
            let bufval = match buf {
                NOOP => NOOP,
                Sampler2D(_, _) => Sampler2D(blob.read_u8(), blob.read_u8()),
            };

            match bufval {
                NOOP => {},
                _ => scene.binding_to_buffers.push(bufval),
            }

            n_buf_mappings -= 1;
        }

        quad_scenes.push(scene);

        n_quad_scenes -= 1;
    }

    // Polygon Context with models

    let mut n_polygon_scenes = blob.read_u8();
    let mut polygon_scenes: SmallVec<[PolygonScene; 64]> = SmallVec::new();

    while n_polygon_scenes >= 1 {
        let mut polygon_scene = PolygonScene::default();

        let mut n_scene_objects = blob.read_u8();

        while n_scene_objects >= 1 {
            let model_idx = blob.read_u8() as usize;

            // position

            let position_var = match code_to_value_vec3(blob.read_u8()) {
                ValueVec3::NOOP => ValueVec3::NOOP,

                ValueVec3::Sync(_, _, _) => ValueVec3::Sync(blob.read_u8(),
                                                            blob.read_u8(),
                                                            blob.read_u8()),
                ValueVec3::Fixed(_, _, _) => ValueVec3::Fixed(blob.read_f32(),
                                                              blob.read_f32(),
                                                              blob.read_f32()),
            };

            // rotation

            let euler_rotation_var = match code_to_value_vec3(blob.read_u8()) {
                ValueVec3::NOOP => ValueVec3::NOOP,

                ValueVec3::Sync(_, _, _) => ValueVec3::Sync(blob.read_u8(),
                                                            blob.read_u8(),
                                                            blob.read_u8()),
                ValueVec3::Fixed(_, _, _) => ValueVec3::Fixed(blob.read_f32(),
                                                              blob.read_f32(),
                                                              blob.read_f32()),
            };

            // scale

            let scale_var = match code_to_value_float(blob.read_u8()) {
                ValueFloat::NOOP => ValueFloat::NOOP,
                ValueFloat::Sync(_) => ValueFloat::Sync(blob.read_u8()),
                ValueFloat::Fixed(_) => ValueFloat::Fixed(blob.read_f32()),
            };

            // layout to vars

            // TODO could simplify with one function doing this here and for quad scenes

            let mut layout_to_vars: SmallVec<[UniformMapping; 64]> = SmallVec::new();

            let mut n_mappings = blob.read_u8();

            while n_mappings >= 1 {
                let uni = code_to_uni(blob.read_u8());

                use dmo::UniformMapping::*;
                let unival = match uni {
                    NOOP => NOOP,

                    Float(_, _) => Float(blob.read_u8(),
                                         blob.read_u8()),

                    Vec2(_, _, _) => Vec2(blob.read_u8(),
                                          blob.read_u8(),
                                          blob.read_u8()),

                    Vec3(_, _, _, _) => Vec3(blob.read_u8(),
                                             blob.read_u8(),
                                             blob.read_u8(),
                                             blob.read_u8()),

                    Vec4(_, _, _, _, _) => Vec4(blob.read_u8(),
                                                blob.read_u8(),
                                                blob.read_u8(),
                                                blob.read_u8(),
                                                blob.read_u8()),

                };

                match unival {
                    NOOP => {},
                    _ => layout_to_vars.push(unival),
                }

                n_mappings -= 1;
            }

            // binding to buffers

            // TODO could simplify with one function doing this here and for quad scenes

            let mut binding_to_buffers: SmallVec<[BufferMapping; 64]> = SmallVec::new();

            let mut n_buf_mappings = blob.read_u8();

            while n_buf_mappings >= 1 {
                let buf = code_to_buf(blob.read_u8());

                use dmo::BufferMapping::*;
                let bufval = match buf {
                    NOOP => NOOP,
                    Sampler2D(_, _) => Sampler2D(blob.read_u8(), blob.read_u8()),
                };

                match bufval {
                    NOOP => {},
                    _ => binding_to_buffers.push(bufval),
                }

                n_buf_mappings -= 1;
            }

            let mut scene_object = SceneObject::default();

            scene_object.model_idx = model_idx;
            scene_object.position_var = position_var;
            scene_object.euler_rotation_var = euler_rotation_var;
            scene_object.scale_var = scale_var;
            scene_object.layout_to_vars = layout_to_vars;
            scene_object.binding_to_buffers = binding_to_buffers;

            polygon_scene.scene_objects.push(scene_object);
            n_scene_objects -= 1;
        }

        polygon_scenes.push(polygon_scene);
        n_polygon_scenes -= 1;
    }

    let mut polygon_context = PolygonContext::default();

    {
        let view_position_var_idx: [usize; 3] = [blob.read_u8() as usize,
                                                 blob.read_u8() as usize,
                                                 blob.read_u8() as usize];

        let view_front_var_idx: [usize; 3] = [blob.read_u8() as usize,
                                              blob.read_u8() as usize,
                                              blob.read_u8() as usize];

        polygon_context.view_position_var_idx = view_position_var_idx;
        polygon_context.view_front_var_idx = view_front_var_idx;
        polygon_context.fovy = blob.read_f32();
        polygon_context.znear = blob.read_f32();
        polygon_context.zfar = blob.read_f32();

        let mut n_models = blob.read_u8();

        while n_models >= 1 {
            let mut model = Model::default();
            model.model_type = code_to_model_type(blob.read_u8());

            let mut n_meshes = blob.read_u8();
            while n_meshes >= 1 {
                let mut mesh = Mesh::default();

                // Read vertex data one at a time

                let mut n_vertices = blob.read_u32();

                while n_vertices >= 1 {
                    let position: [f32; 3] = [blob.read_f32(),
                                              blob.read_f32(),
                                              blob.read_f32()];

                    let normal: [f32; 3] = [blob.read_f32(),
                                            blob.read_f32(),
                                            blob.read_f32()];

                    // println!("{:>3} pos: {:>5.1} {:>5.1} {:>5.1}   nor: {:>5.1} {:>5.1} {:>5.1}",
                    //          n_vertices,
                    //          position[0],
                    //          position[1],
                    //          position[2],
                    //          normal[0],
                    //          normal[1],
                    //          normal[2]);

                    let vertex = Vertex {
                        position: position,
                        normal: normal,
                        texcoords: [0.0; 2], // TODO UV texcoords
                    };

                    mesh.vertices.push(vertex);

                    n_vertices -= 1;
                }

                mesh.vert_src_idx = blob.read_u8() as usize;
                mesh.frag_src_idx = blob.read_u8() as usize;

                model.meshes.push(mesh);
                n_meshes -= 1;
            }

            polygon_context.models.push(model);
            n_models -= 1;
        }
    }

    let context = Context::new(0.0,
                               window_width,
                               window_height,
                               screen_width,
                               screen_height,
                               shader_sources,
                               images,
                               quad_scenes,
                               polygon_scenes,
                               polygon_context,
                               frame_buffers);

    // === Operators ===

    let mut operators: SmallVec<[Operator; 64]> = SmallVec::new();

    let mut n_operators = blob.read_u8();

    while n_operators >= 1 {
        let op = code_to_op(blob.read_u8());

        use dmo::Operator::*;
        let opval = match op {
            NOOP => NOOP,

            Exit(_) => Exit(blob.read_f64()),

            Draw_Quad_Scene(_) => Draw_Quad_Scene(blob.read_u8()),

            If_Var_Equal_Draw_Quad(_, _, _) => If_Var_Equal_Draw_Quad(blob.read_u8(),
                                                                      blob.read_f64(),
                                                                      blob.read_u8()),

            If_Var_Equal_Draw_Polygon(_, _, _) => If_Var_Equal_Draw_Polygon(blob.read_u8(),
                                                                            blob.read_f64(),
                                                                            blob.read_u8()),

            Clear(_, _, _, _) => Clear(blob.read_u8(),
                                       blob.read_u8(),
                                       blob.read_u8(),
                                       blob.read_u8()),

            Target_Buffer(_) => Target_Buffer(blob.read_u8()),

            Target_Buffer_Default => Target_Buffer_Default,

            Profile_Event(_) => Profile_Event(blob.read_u8()),
        };

        match opval {
            NOOP => {},
            _ => operators.push(opval),
        }

        n_operators -= 1;
    }

    // === Sync ===

    let bpm: f64 = blob.read_f64();
    let rpb: u8 = blob.read_u8();

    let mut device: SyncDevice = SyncDevice::new(bpm, rpb);

    let mut n_tracks = blob.read_u8();

    while n_tracks >= 1 {

        let mut track: SyncTrack = SyncTrack::new();
        let mut n_keys = blob.read_u32();

        while n_keys >= 1 {

            let key = TrackKey{
                row: blob.read_u32(),
                value: blob.read_f32(),
                key_type: code_to_key(blob.read_u8()),
            };

            track.add_key(key);
            n_keys -= 1;
        }

        device.tracks.push(track);
        n_tracks -= 1;
    }

    // sync operators

    let mut n_sync_ops = blob.read_u8();

    let mut sync_ops: SmallVec<[SyncOp; 64]> = SmallVec::new();

    while n_sync_ops >= 1 {
        let op = code_to_syncop(blob.read_u8());

        use self::SyncOp::*;
        let opval = match op {
            NOOP => NOOP,

            Time_Var => Time_Var,

            Track_To_Var(_, _) => Track_To_Var(blob.read_u8(),
                                               blob.read_u8()),
        };

        match opval {
            NOOP => {},
            _ => sync_ops.push(opval),
        }

        n_sync_ops -= 1;
    }

    let sync = DmoSync{
        device: device,
        ops: sync_ops,
    };

    Dmo::new(context, operators, sync)
}

pub fn op_to_code(op: Operator) -> u8 {
    use dmo::Operator::*;
    match op {
        NOOP                               => 0x00,
        Exit(_)                            => 0x01,
        Draw_Quad_Scene(_)                 => 0x02,
        Clear(_, _, _, _)                  => 0x03,
        Target_Buffer(_)                   => 0x04,
        Target_Buffer_Default              => 0x05,
        If_Var_Equal_Draw_Quad(_, _, _)    => 0x06,
        If_Var_Equal_Draw_Polygon(_, _, _) => 0x07,
        Profile_Event(_)                   => 0x08,
    }
}

pub fn code_to_op(code: u8) -> Operator {
    use dmo::Operator::*;
    match code {
        0x00 => NOOP,
        0x01 => Exit(0.0),
        0x02 => Draw_Quad_Scene(0),
        0x03 => Clear(0, 0, 0, 0),
        0x04 => Target_Buffer(0),
        0x05 => Target_Buffer_Default,
        0x06 => If_Var_Equal_Draw_Quad(0, 0.0, 0),
        0x07 => If_Var_Equal_Draw_Polygon(0, 0.0, 0),
        0x08 => Profile_Event(0),
        _ => NOOP,
    }
}

pub fn uni_to_code(uni: UniformMapping) -> u8 {
    use dmo::UniformMapping::*;
    match uni {
        NOOP                 => 0x00,
        Float(_, _)          => 0x01,
        Vec2(_, _, _)        => 0x02,
        Vec3(_, _, _, _)     => 0x03,
        Vec4(_, _, _, _, _)  => 0x04,
    }
}

pub fn code_to_uni(code: u8) -> UniformMapping {
    use dmo::UniformMapping::*;
    match code {
        0x00 => NOOP,
        0x01 => Float(0, 0),
        0x02 => Vec2(0, 0, 0),
        0x03 => Vec3(0, 0, 0, 0),
        0x04 => Vec4(0, 0, 0, 0, 0),
        _ => NOOP,
    }
}

pub fn syncop_to_code(syncop: SyncOp) -> u8 {
    use sync::SyncOp::*;
    match syncop {
        NOOP                            => 0x00,
        Time_Var                        => 0x01,
        Track_To_Var(_, _)              => 0x02,
    }
}

pub fn code_to_syncop(code: u8) -> SyncOp {
    use sync::SyncOp::*;
    match code {
        0x00 => NOOP,
        0x01 => Time_Var,
        0x02 => Track_To_Var(0, 0),
        _ => NOOP,
    }
}

pub fn buf_to_code(buf: BufferMapping) -> u8 {
    use dmo::BufferMapping::*;
    match buf {
        NOOP            => 0x00,
        Sampler2D(_, _) => 0x01,
    }
}

pub fn code_to_buf(code: u8) -> BufferMapping {
    use dmo::BufferMapping::*;
    match code {
        0x00 => NOOP,
        0x01 => Sampler2D(0, 0),
        _ => NOOP,
    }
}

pub fn bufkind_to_code(kind: BufferKind) -> u8 {
    use dmo::BufferKind::*;
    match kind {
        NOOP    => 0x00,
        Empty_Texture => 0x01,
        Image_Texture => 0x02,
    }
}

pub fn code_to_bufkind(code: u8) -> BufferKind {
    use dmo::BufferKind::*;
    match code {
        0x00 => NOOP,
        0x01 => Empty_Texture,
        0x02 => Image_Texture,
        _ => NOOP,
    }
}

pub fn format_to_code(format: PixelFormat) -> u8 {
    use dmo::PixelFormat::*;
    match format {
        NOOP =>     0x00,
        RED_u8 =>   0x01,
        RGB_u8 =>   0x02,
        RGBA_u8 =>  0x03,
    }
}

pub fn code_to_format(code: u8) -> PixelFormat {
    use dmo::PixelFormat::*;
    match code {
        0x00 => NOOP,
        0x01 => RED_u8,
        0x02 => RGB_u8,
        0x03 => RGBA_u8,
        _ => NOOP,
    }
}

pub fn model_type_to_code(model_type: ModelType) -> u8 {
    use dmo::ModelType::*;
    match model_type {
        NOOP => 0x00,
        Cube => 0x01,
        Obj =>  0x02,
    }
}

pub fn code_to_model_type(code: u8) -> ModelType {
    use dmo::ModelType::*;
    match code {
        0x00 => NOOP,
        0x01 => Cube,
        0x02 => Obj,
        _ => NOOP,
    }
}

pub fn value_vec3_to_code(value_vec3: ValueVec3) -> u8 {
    use dmo::ValueVec3::*;
    match value_vec3 {
        NOOP           => 0x00,
        Sync(_, _, _)  => 0x01,
        Fixed(_, _, _) => 0x02,
    }
}

pub fn code_to_value_vec3(code: u8) -> ValueVec3 {
    use dmo::ValueVec3::*;
    match code {
        0x00 => NOOP,
        0x01 => Sync(0, 0, 0),
        0x02 => Fixed(0.0, 0.0, 0.0),
        _ => NOOP,
    }
}

pub fn value_float_to_code(value_float: ValueFloat) -> u8 {
    use dmo::ValueFloat::*;
    match value_float {
        NOOP     => 0x00,
        Sync(_)  => 0x01,
        Fixed(_) => 0x02,
    }
}

pub fn code_to_value_float(code: u8) -> ValueFloat {
    use dmo::ValueFloat::*;
    match code {
        0x00 => NOOP,
        0x01 => Sync(0),
        0x02 => Fixed(0.0),
        _ => NOOP,
    }
}

pub struct DataBlob {
    data: SmallVec<[u8; 0x8000]>,
    idx: usize,
}

impl DataBlob {
    pub fn new(data: SmallVec<[u8; 0x8000]>) -> DataBlob {
        DataBlob {
            data: data,
            idx: 0,
        }
    }

    pub fn get_idx(&self) -> usize {
        self.idx
    }

    pub fn skip(&mut self, skip_len: usize) {
        self.idx += skip_len
    }

    pub fn read_u8(&mut self) -> u8 {
        let number = self.data[self.idx];
        self.idx += 1;
        number
    }

    pub fn read_u16(&mut self) -> u16 {
        let bytes: &[u8] = &self.data[self.idx .. self.idx+2];

        let mut number: u16 = 0;
        unsafe {
            ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                &mut number as *mut u16 as *mut u8,
                2);
        };
        number.to_le();

        self.idx += 2;
        number
    }

    pub fn read_u32(&mut self) -> u32 {
        let bytes: &[u8] = &self.data[self.idx .. self.idx+4];

        let mut number: u32 = 0;
        unsafe {
            ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                &mut number as *mut u32 as *mut u8,
                4);
        };
        number.to_le();

        self.idx += 4;
        number
    }

    pub fn read_u64(&mut self) -> u64 {
        let bytes: &[u8] = &self.data[self.idx .. self.idx+8];

        let mut number: u64 = 0;
        unsafe {
            ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                &mut number as *mut u64 as *mut u8,
                8);
        };
        number.to_le();

        self.idx += 8;
        number
    }

    pub fn read_f32(&mut self) -> f32 {
        let number: f32 = unsafe { mem::transmute(self.read_u32()) };
        number
    }

    pub fn read_f64(&mut self) -> f64 {
        let number: f64 = unsafe { mem::transmute(self.read_u64()) };
        number
    }

    pub fn read_str(&mut self, str_len: usize) -> &str {
        if str_len == 0 {
            return "";
        }
        let text = str::from_utf8(&self.data[self.idx .. self.idx+str_len]).unwrap();
        self.idx += str_len;
        text
    }

    pub fn read_u8_vec(&mut self, len: usize) -> SmallVec<[u8; 1024]> {
        let mut ret: SmallVec<[u8; 1024]> = SmallVec::new();

        ret.extend(self.data[self.idx .. self.idx+len].iter().cloned());

        self.idx += len;
        ret
    }

    pub fn read_f32_vec(&mut self, len: usize) -> SmallVec<[f32; 1024]> {
        let mut ret: SmallVec<[f32; 1024]> = SmallVec::new();

        for _ in 0 .. len {
            ret.push(self.read_f32());
        }

        ret
    }
}

pub fn push_u32(v: &mut SmallVec<[u8; 64]>, n: u32) {
    let bytes = unsafe { mem::transmute::<_, [u8; 4]>(n.to_le()) };
    v.push(bytes[0]);
    v.push(bytes[1]);
    v.push(bytes[2]);
    v.push(bytes[3]);
}

pub fn push_f32(v: &mut SmallVec<[u8; 64]>, n: f32) {
    let val_u32: u32 = unsafe { mem::transmute(n) };
    push_u32(v, val_u32);
}

// NOTE: read_num_bytes and write_num_bytes macro in the byteorder crate by
// BurntSushi
//
// macro_rules! read_num_bytes {
//     ($ty:ty, $size:expr, $src:expr, $which:ident) => ({
//         assert!($size == ::core::mem::size_of::<$ty>());
//         assert!($size <= $src.len());
//         let mut data: $ty = 0;
//         unsafe {
//             copy_nonoverlapping(
//                 $src.as_ptr(),
//                 &mut data as *mut $ty as *mut u8,
//                 $size);
//         }
//         data.$which()
//     });
// }
//
// macro_rules! write_num_bytes {
//     ($ty:ty, $size:expr, $n:expr, $dst:expr, $which:ident) => ({
//         assert!($size <= $dst.len());
//         unsafe {
//             // N.B. https://github.com/rust-lang/rust/issues/22776
//             let bytes = transmute::<_, [u8; $size]>($n.$which());
//             copy_nonoverlapping((&bytes).as_ptr(), $dst.as_mut_ptr(), $size);
//         }
//     });
// }
