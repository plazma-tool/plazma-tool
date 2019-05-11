use std::error::Error;
use std::path::PathBuf;

use crate::dmo_data::quad_scene::QuadScene;
use crate::dmo_data::polygon_scene::PolygonScene;
use crate::dmo_data::polygon_context::PolygonContext;
use crate::dmo_data::data_index::DataIndex;

#[derive(Serialize, Deserialize, Debug)]
pub struct ContextData {

    /// Do both serialize and deserialize on `.shader_sources[]`.
    ///
    /// Deserializing from YAML file: shader sources will have to be declared in the file, but can
    /// be left empty. The paths will be read into the sources array:
    ///
    /// - the YAML is parsed
    /// - shader sources are read from path and added to `.shader_sources[]`
    /// - path to index mapping is stored in `.index`
    ///
    /// Deserializing from server: `.shader_sources[]` will be used as the server is sending it,
    /// paths are not read again. Otherwise the shader content on the OpenGL client wouldn't
    /// change, sinces the shaders are only edited in the browser, not saved back to file.
    ///
    /// Serializing it is also necessary for sending DmoData to the browser UI and OpenGL preview
    /// client, so that the index mapping between server and client can stay the same.
    pub shader_sources: Vec<String>,

    #[serde(skip_serializing, skip_deserializing)]
    pub image_sources: Vec<Image>,

    pub frame_buffers: Vec<FrameBuffer>,

    pub quad_scenes: Vec<QuadScene>,

    pub polygon_scenes: Vec<PolygonScene>,
    pub polygon_context: PolygonContext,

    // TODO pub audio_path: PathBuf,

    pub sync_tracks_path: String,

    /// Do serialize, so that paths and array index data can be used on the server.
    ///
    /// Don't deserialize, the index doesn't have to be included in the YAML file and doesn't have to be
    /// sent by the server to the client.
    ///
    /// When a new shader file is added in the browser UI, a specific message will be send, the
    /// shader read in and the data sent back to the browser.
    #[serde(skip_deserializing)]
    pub index: DataIndex,
}

impl ContextData {
    pub fn build_index(&mut self,
                       project_root: &Option<PathBuf>,
                       read_shader_paths: bool,
                       read_image_paths: bool)
        -> Result<(), Box<Error>>
    {
        // First, empty any existing index data.
        self.index = DataIndex::new();

        for (idx, scene) in self.quad_scenes.iter_mut().enumerate() {
            self.index.add_quad_scene(scene, idx, project_root, read_shader_paths, &mut self.shader_sources)?;
        }

        for (idx, buffer) in self.frame_buffers.iter().enumerate() {
            self.index.add_frame_buffer(buffer, idx, project_root, read_image_paths, &mut self.image_sources)?;
        }

        for (idx, model) in self.polygon_context.models.iter().enumerate() {
            self.index.add_model(model, idx, project_root, read_shader_paths, &mut self.shader_sources)?;
        }

        for (idx, scene) in self.polygon_scenes.iter().enumerate() {
            self.index.add_polygon_scene(scene, idx)?;
        }

        Ok(())
    }
}

impl Default for ContextData {
    fn default() -> ContextData {
        ContextData {
            shader_sources: vec![],
            image_sources: vec![],
            frame_buffers: vec![],
            quad_scenes: vec![],
            polygon_scenes: vec![],
            polygon_context: PolygonContext::default(),
            //audio_path: PathBuf::from(""),
            sync_tracks_path: String::new(),
            index: DataIndex::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FrameBuffer {
    pub name: String,
    pub kind: BufferKind,
    pub format: PixelFormat,
    pub image_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub raw_pixels: Vec<u8>,
}

impl FrameBuffer {
    pub fn framebuffer_result_image() -> FrameBuffer {
        FrameBuffer{
            name: "RESULT_IMAGE".to_string(),
            kind: BufferKind::Empty_Texture,
            format: PixelFormat::RGBA_u8,
            image_path: "".to_owned(),
        }
    }
}

/// Specifies the frame buffer kind to be generated
#[derive(Serialize, Deserialize, Debug)]
pub enum BufferKind {
    NOOP,
    Empty_Texture,
    Image_Texture,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum PixelFormat {
    NOOP,
    RED_u8,
    RGB_u8,
    RGBA_u8,
}

