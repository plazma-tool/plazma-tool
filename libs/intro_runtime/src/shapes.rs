pub static QUAD_VERTICES: [f32; 16] = [
    // pos: x, y, tex: u, v
    -1.0, -1.0, 0.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0,
];

// FIXME verify tex coords and normals
pub static CUBE_VERTICES: [[f32; 3 + 3 + 2]; 4 * 6] = [
    // position x,y,z,     normal x,y,z,       tex coord x,y
    // front
    [-1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    [1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
    [1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    [-1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0],
    // top
    [-1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0],
    [1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
    [1.0, 1.0, -1.0, 0.0, 1.0, 0.0, 1.0, 1.0],
    [-1.0, 1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 1.0],
    // back
    [1.0, -1.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    [-1.0, -1.0, -1.0, 0.0, 0.0, -1.0, 1.0, 0.0],
    [-1.0, 1.0, -1.0, 0.0, 0.0, -1.0, 1.0, 1.0],
    [1.0, 1.0, -1.0, 0.0, 0.0, -1.0, 0.0, 1.0],
    // bottom
    [-1.0, -1.0, -1.0, 0.0, -1.0, 0.0, 0.0, 0.0],
    [1.0, -1.0, -1.0, 0.0, -1.0, 0.0, 1.0, 0.0],
    [1.0, -1.0, 1.0, 0.0, -1.0, 0.0, 1.0, 1.0],
    [-1.0, -1.0, 1.0, 0.0, -1.0, 0.0, 0.0, 1.0],
    // left
    [-1.0, -1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 0.0],
    [-1.0, -1.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0],
    [-1.0, 1.0, 1.0, -1.0, 0.0, 0.0, 1.0, 1.0],
    [-1.0, 1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0],
    // right
    [1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    [1.0, -1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 0.0],
    [1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0],
];

pub static CUBE_ELEMENTS: [u32; 3 * 2 * 6] = [
    // front
    0, 1, 2, 2, 3, 0, // top
    4, 5, 6, 6, 7, 4, // back
    8, 9, 10, 10, 11, 8, // bottom
    12, 13, 14, 14, 15, 12, // left
    16, 17, 18, 18, 19, 16, // right
    20, 21, 22, 22, 23, 20,
];
