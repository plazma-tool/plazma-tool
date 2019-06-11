use std::error::Error;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::project_data::{get_template_asset_string, get_template_asset_bytes};
use crate::dmo_data::context_data::{FrameBuffer, Image, PixelFormat};
use crate::dmo_data::quad_scene::QuadScene;
use crate::dmo_data::quad_scene::{DRAW_RESULT_VERT_SRC_PATH, DRAW_RESULT_FRAG_SRC_PATH};
use crate::dmo_data::polygon_scene::PolygonScene;
use crate::dmo_data::model::Model;
use crate::error::ToolError;
use crate::utils::file_to_string;

use image::{self, GenericImageView};

#[derive(Serialize, Debug)]
pub struct DataIndex {
    /// Private to ensure unique paths by an adder function.
    shader_paths: Vec<String>,
    shader_path_to_idx: BTreeMap<String, usize>,

    image_paths: Vec<String>,
    image_path_to_idx: BTreeMap<String, usize>,
    image_path_to_format: BTreeMap<String, PixelFormat>,

    pub obj_paths: Vec<String>,
    pub quad_scene_name_to_idx: BTreeMap<String, usize>,
    pub polygon_scene_name_to_idx: BTreeMap<String, usize>,
    pub model_name_to_idx: BTreeMap<String, usize>,
    pub obj_path_to_idx: BTreeMap<String, usize>,
    pub buffer_name_to_idx: BTreeMap<String, usize>,
}

impl DataIndex {
    pub fn new() -> DataIndex {
        DataIndex {
            shader_paths: vec![],
            image_paths: vec![],
            obj_paths: vec![],
            quad_scene_name_to_idx: BTreeMap::new(),
            polygon_scene_name_to_idx: BTreeMap::new(),
            model_name_to_idx: BTreeMap::new(),
            shader_path_to_idx: BTreeMap::new(),
            obj_path_to_idx: BTreeMap::new(),
            buffer_name_to_idx: BTreeMap::new(),
            image_path_to_idx: BTreeMap::new(),
            image_path_to_format: BTreeMap::new(),
        }
    }

    pub fn add_quad_scene(&mut self,
                          scene: &QuadScene,
                          idx: usize,
                          project_root: &Option<PathBuf>,
                          read_shader_paths: bool,
                          shader_sources: &mut Vec<String>,
                          embedded: bool)
        -> Result<(), Box<dyn Error>>
    {
        if self.quad_scene_name_to_idx.contains_key(&scene.name) {
            return Err(Box::new(ToolError::NameAlreadyExists));
        }

        self.quad_scene_name_to_idx.insert(scene.name.to_string(), idx);
        self.add_shader(&scene.vert_src_path, project_root, read_shader_paths, shader_sources, embedded)?;
        self.add_shader(&scene.frag_src_path, project_root, read_shader_paths, shader_sources, embedded)?;

        Ok(())
    }

    pub fn add_polygon_scene(&mut self,
                             scene: &PolygonScene,
                             idx: usize)
                             -> Result<(), Box<dyn Error>>
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
                     project_root: &Option<PathBuf>,
                     read_shader_paths: bool,
                     shader_sources: &mut Vec<String>,
                     embedded: bool)
        -> Result<(), Box<dyn Error>>
    {
        if self.model_name_to_idx.contains_key(&model.name) {
            return Err(Box::new(ToolError::NameAlreadyExists));
        }

        self.model_name_to_idx.insert(model.name.to_string(), idx);

        self.add_shader(&model.vert_src_path, project_root, read_shader_paths, shader_sources, embedded)?;
        self.add_shader(&model.frag_src_path, project_root, read_shader_paths, shader_sources, embedded)?;

        Ok(())
    }

    pub fn add_shader(&mut self,
                      path: &str,
                      project_root: &Option<PathBuf>,
                      read_shader_path: bool,
                      shader_sources: &mut Vec<String>,
                      embedded: bool)
        -> Result<(), Box<dyn Error>>
    {
        info!{"add_shader() path: {}, embedded {}", path, embedded};
        // TODO send error (which can be ignored) when path length is zero.
        if path.len() == 0 {
            return Ok(());
        }

        if self.shader_path_to_idx.contains_key(path) {
            return Ok(());
        }

        self.add_shader_path_to_index(path);

        if read_shader_path
            && path != DRAW_RESULT_VERT_SRC_PATH
            && path != DRAW_RESULT_FRAG_SRC_PATH
        {
            let p: PathBuf = if let Some(project_root) = project_root {
                project_root.join(PathBuf::from(path))
            } else {
                return Err(Box::new(ToolError::MissingProjectRoot));
            };
            let src = if embedded {
                get_template_asset_string(&p)?
            } else {
                file_to_string(&p)?
            };
            shader_sources.push(src.to_owned());
        }

        return Ok(());
    }

    pub fn add_frame_buffer(&mut self,
                            buffer: &FrameBuffer,
                            idx: usize,
                            project_root: &Option<PathBuf>,
                            read_image_path: bool,
                            image_sources: &mut Vec<Image>,
                            embedded: bool)
        -> Result<(), Box<dyn Error>>
        {

        if self.buffer_name_to_idx.contains_key(&buffer.name) {
            return Err(Box::new(ToolError::NameAlreadyExists));
        }

        self.buffer_name_to_idx.insert(buffer.name.to_string(), idx);

        // TODO should error if buffer is not Empty_Texture but path.len() == 0
        if buffer.image_path.len() > 0 {
            self.add_image_path_format_to_index(&buffer.image_path, buffer.format.clone());

            if read_image_path {
                let p: PathBuf = if let Some(project_root) = project_root {
                    project_root.join(PathBuf::from(&buffer.image_path))
                } else {
                    return Err(Box::new(ToolError::MissingProjectRoot));
                };
                let image_data = if embedded {
                    image::load_from_memory(&get_template_asset_bytes(&p)?)?
                } else {
                    image::open(&p)?
                };
                let (width, height) = image_data.dimensions();

                let f = self.image_path_to_format.get(&buffer.image_path.clone()).ok_or("bad image path name")?;
                let mut new_image = Image {
                    width: width,
                    height: height,
                    format: *f,
                    raw_pixels: vec![],
                };

                for x in image_data.raw_pixels().iter() {
                    new_image.raw_pixels.push(*x);
                }

                image_sources.push(new_image);
            }
        }

        Ok(())
    }

    pub fn add_image_path_format_to_index(&mut self, path: &str, format: PixelFormat) {
        // Ensure path doesn't already exist. The BTreeMap would just overwrite
        // it, but the `image_paths[]` would accumulate duplicates.
        if self.image_path_to_idx.contains_key(path) {
            return;
        }

        self.image_paths.push(path.to_owned());
        let idx = self.image_paths.len() - 1;
        self.image_path_to_idx.insert(path.to_owned(), idx);

        self.image_path_to_format.insert(path.to_owned(), format);
    }

    pub fn add_shader_path_to_index(&mut self, path: &str) {
        // Ensure path doesn't already exist. The BTreeMap would just overwrite
        // it, but the `shader_paths[]` would accumulate duplicates.
        if self.shader_path_to_idx.contains_key(path) {
            return;
        }

        self.shader_paths.push(path.to_owned());
        let idx = self.shader_paths.len() - 1;
        self.shader_path_to_idx.insert(path.to_owned(), idx);
    }

    pub fn get_shader_index(&self, path: &str) -> Result<usize, Box<dyn Error>> {
        let idx = self.shader_path_to_idx.get(path).ok_or(format!{"no such shader path: {}", path})?;
        Ok(*idx)
    }

    pub fn get_shader_path_to_idx(&self) -> BTreeMap<String, usize> {
        self.shader_path_to_idx.clone()
    }

    pub fn get_image_index(&self, path: &str) -> Result<usize, Box<dyn Error>> {
        let idx = self.image_path_to_idx.get(path).ok_or(format!{"no such image path: {}", path})?;
        Ok(*idx)
    }

    pub fn get_buffer_index(&self, name: &str) -> Result<usize, Box<dyn Error>> {
        let idx = self.buffer_name_to_idx.get(name).ok_or(format!{"no such buffer name: {}", name})?;
        Ok(*idx)
    }

    pub fn get_quad_scene_index(&self, name: &str) -> Result<usize, Box<dyn Error>> {
        let idx = self.quad_scene_name_to_idx.get(name).ok_or(format!{"no such quad scene name: {}", name})?;
        Ok(*idx)
    }

    pub fn get_polygon_scene_index(&self, name: &str) -> Result<usize, Box<dyn Error>> {
        let idx = self.polygon_scene_name_to_idx.get(name).ok_or(format!{"no such polygon scene name: {}", name})?;
        Ok(*idx)
    }

    pub fn get_model_index(&self, name: &str) -> Result<usize, Box<dyn Error>> {
        let idx = self.model_name_to_idx.get(name).ok_or(format!{"no such model name: {}", name})?;
        Ok(*idx)
    }

    pub fn get_profile_index(&self, _name: &str) -> Result<usize, Box<dyn Error>> {
        // TODO implement
        Ok(0)
    }
}

impl Default for DataIndex {
    fn default() -> DataIndex {
        DataIndex::new()
    }
}
