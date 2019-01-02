use smallvec::SmallVec;

use gl::types::*;

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

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub raw_pixels: SmallVec<[u8; 1024]>,
}

#[derive(Copy, Clone)]
pub enum PixelFormat {
    NOOP,
    RED_u8,
    RGB_u8,
    RGBA_u8,
}

/// Value for a `vec3` type uniform. Either sync `.xyz` from tracks, or set a
/// fixed value.
pub enum ValueVec3 {
    NOOP,
    Sync(u8, u8, u8),
    Fixed(f32, f32, f32),
}

/// Value for a `float` type uniform. Either sync from a track, or set a fixed
/// value.
pub enum ValueFloat {
    NOOP,
    Sync(u8),
    Fixed(f32),
}

/// Map a uniform type: (layout_idx, vars idx...)
pub enum UniformMapping {
    NOOP,
    Float(u8, u8),
    Vec2(u8, u8, u8),
    Vec3(u8, u8, u8, u8),
    Vec4(u8, u8, u8, u8, u8),
}

/// Map a frame buffer: (layout_idx, buffer_idx)
pub enum BufferMapping {
    NOOP,
    Sampler2D(u8, u8),
}
