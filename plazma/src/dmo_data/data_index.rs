use std::error::Error;
use std::collections::BTreeMap;

use crate::dmo_data::context_data::FrameBuffer;
use crate::dmo_data::quad_scene::QuadScene;

#[derive(Debug)]
pub struct DataIndex {
    pub shader_paths: Vec<String>,
    //pub obj_paths: Vec<String>,
    //pub image_paths: Vec<String>,
    pub quad_scene_name_to_idx: BTreeMap<String, usize>,
    //pub polygon_scene_name_to_idx: BTreeMap<String, u8>,
    //pub model_name_to_idx: BTreeMap<String, u8>,
    pub shader_path_to_idx: BTreeMap<String, usize>,
    //pub obj_path_to_idx: BTreeMap<String, u8>,
    //pub image_path_to_idx: BTreeMap<String, u8>,
    //pub image_path_to_format: BTreeMap<String, TrPixelFormat>,
    pub buffer_name_to_idx: BTreeMap<String, usize>,
}

impl DataIndex {
    pub fn new() -> DataIndex {
        DataIndex {
            shader_paths: vec![],
            quad_scene_name_to_idx: BTreeMap::new(),
            shader_path_to_idx: BTreeMap::new(),
            buffer_name_to_idx: BTreeMap::new(),
        }
    }

    pub fn add_quad_scene(&mut self, scene: &QuadScene, idx: usize) {
        self.quad_scene_name_to_idx.insert(scene.name.clone(), idx);
        self.add_shader(&scene.vert_src_path);
        self.add_shader(&scene.frag_src_path);
    }

    pub fn add_shader(&mut self, path: &str) {
        self.shader_paths.push(path.to_string());
        let idx = self.shader_paths.len() - 1;
        self.shader_path_to_idx.insert(path.to_string(), idx);
    }

    pub fn add_frame_buffer(&mut self, buffer: &FrameBuffer, idx: usize) {
        self.buffer_name_to_idx.insert(buffer.name.to_string(), idx);
    }

    pub fn get_buffer_index(&self, name: &str) -> Result<usize, Box<Error>> {
        let idx = self.buffer_name_to_idx.get(name).ok_or("cannot get index: bad buffer name")?;
        Ok(*idx)
    }
}

impl Default for DataIndex {
    fn default() -> DataIndex {
        DataIndex::new()
    }
}
