use std::error::Error;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::dmo_data::context_data::FrameBuffer;
use crate::dmo_data::quad_scene::QuadScene;
use crate::dmo_data::polygon_scene::PolygonScene;
use crate::dmo_data::model::Model;
use crate::error::ToolError;
use crate::utils::file_to_string;

#[derive(Serialize, Debug)]
pub struct DataIndex {
    /// Private to ensure unique paths by an adder function.
    shader_paths: Vec<String>,
    shader_path_to_idx: BTreeMap<String, usize>,

    pub obj_paths: Vec<String>,
    //pub image_paths: Vec<String>,
    pub quad_scene_name_to_idx: BTreeMap<String, usize>,
    pub polygon_scene_name_to_idx: BTreeMap<String, usize>,
    pub model_name_to_idx: BTreeMap<String, usize>,
    pub obj_path_to_idx: BTreeMap<String, usize>,
    //pub image_path_to_idx: BTreeMap<String, u8>,
    //pub image_path_to_format: BTreeMap<String, TrPixelFormat>,
    pub buffer_name_to_idx: BTreeMap<String, usize>,
}

impl DataIndex {
    pub fn new() -> DataIndex {
        DataIndex {
            shader_paths: vec![],
            obj_paths: vec![],
            quad_scene_name_to_idx: BTreeMap::new(),
            polygon_scene_name_to_idx: BTreeMap::new(),
            model_name_to_idx: BTreeMap::new(),
            shader_path_to_idx: BTreeMap::new(),
            obj_path_to_idx: BTreeMap::new(),
            buffer_name_to_idx: BTreeMap::new(),
        }
    }

    pub fn add_quad_scene(&mut self,
                          scene: &QuadScene,
                          idx: usize,
                          read_shader_paths: bool,
                          shader_sources: &mut Vec<String>)
        -> Result<(), Box<Error>>
    {
        if self.quad_scene_name_to_idx.contains_key(&scene.name) {
            return Err(Box::new(ToolError::NameAlreadyExists));
        }

        self.quad_scene_name_to_idx.insert(scene.name.to_string(), idx);
        self.add_shader(&scene.vert_src_path, read_shader_paths, shader_sources)?;
        self.add_shader(&scene.frag_src_path, read_shader_paths, shader_sources)?;

        Ok(())
    }

    pub fn add_polygon_scene(&mut self,
                             scene: &PolygonScene,
                             idx: usize)
                             -> Result<(), Box<Error>>
    {
        if self.polygon_scene_name_to_idx.contains_key(&scene.name) {
            return Err(Box::new(ToolError::NameAlreadyExists));
        }

        self.polygon_scene_name_to_idx.insert(scene.name.to_string(), idx);

        Ok(())
    }

    pub fn add_model(&mut self,
                     model: &Model,
                     idx: usize,
                     read_shader_paths: bool,
                     shader_sources: &mut Vec<String>)
        -> Result<(), Box<Error>>
    {
        if self.model_name_to_idx.contains_key(&model.name) {
            return Err(Box::new(ToolError::NameAlreadyExists));
        }

        self.model_name_to_idx.insert(model.name.to_string(), idx);

        self.add_shader(&model.vert_src_path, read_shader_paths, shader_sources)?;
        self.add_shader(&model.frag_src_path, read_shader_paths, shader_sources)?;

        Ok(())
    }

    pub fn add_shader(&mut self,
                      path: &str,
                      read_shader_path: bool,
                      shader_sources: &mut Vec<String>)
        -> Result<(), Box<Error>>
    {
        // TODO send error (which can be ignored) when path length is zero.
        if path.len() == 0 {
            return Ok(());
        }

        if self.shader_path_to_idx.contains_key(path) {
            return Ok(());
        }

        self.add_shader_path_and_index(path);

        if read_shader_path {
            let src = file_to_string(&PathBuf::from(path))?;
            shader_sources.push(src.to_owned());
        }

        return Ok(());
    }

    pub fn add_frame_buffer(&mut self, buffer: &FrameBuffer, idx: usize) -> Result<(), Box<Error>> {
        if self.buffer_name_to_idx.contains_key(&buffer.name) {
            return Err(Box::new(ToolError::NameAlreadyExists));
        }

        self.buffer_name_to_idx.insert(buffer.name.to_string(), idx);

        Ok(())
    }

    pub fn add_shader_path_and_index(&mut self, path: &str) {
        // Ensure path doesn't already exist. The BTreeMap would just overwrite
        // it, but the `shader_paths[]` would accumulate duplicates.
        if self.shader_path_to_idx.contains_key(path) {
            return;
        }

        self.shader_paths.push(path.to_owned());
        let idx = self.shader_paths.len() - 1;
        self.shader_path_to_idx.insert(path.to_owned(), idx);
    }

    pub fn get_shader_index(&self, path: &str) -> Result<usize, Box<Error>> {
        let idx = self.shader_path_to_idx.get(path).ok_or(format!{"no such shader path: {}", path})?;
        Ok(*idx)
    }

    pub fn get_buffer_index(&self, name: &str) -> Result<usize, Box<Error>> {
        let idx = self.buffer_name_to_idx.get(name).ok_or(format!{"no such buffer name: {}", name})?;
        Ok(*idx)
    }

    pub fn get_quad_scene_index(&self, name: &str) -> Result<usize, Box<Error>> {
        let idx = self.quad_scene_name_to_idx.get(name).ok_or(format!{"no such quad scene name: {}", name})?;
        Ok(*idx)
    }

    pub fn get_polygon_scene_index(&self, name: &str) -> Result<usize, Box<Error>> {
        let idx = self.polygon_scene_name_to_idx.get(name).ok_or(format!{"no such polygon scene name: {}", name})?;
        Ok(*idx)
    }

    pub fn get_model_index(&self, name: &str) -> Result<usize, Box<Error>> {
        let idx = self.model_name_to_idx.get(name).ok_or(format!{"no such model name: {}", name})?;
        Ok(*idx)
    }

    pub fn get_profile_index(&self, _name: &str) -> Result<usize, Box<Error>> {
        // TODO implement
        Ok(0)
    }
}

impl Default for DataIndex {
    fn default() -> DataIndex {
        DataIndex::new()
    }
}
