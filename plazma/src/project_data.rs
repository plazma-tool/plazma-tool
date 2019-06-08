use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::error::Error;
use std::borrow::Cow;

use crate::utils::file_to_string;
use crate::dmo_data::DmoData;
use crate::error::ToolError;

pub struct ProjectData {
    pub project_root: Option<PathBuf>,
    pub demo_yml_path: Option<PathBuf>,
    pub dmo_data: DmoData,
    pub embedded: bool,
}

#[derive(RustEmbed)]
#[folder = "./data/templates/"]
pub struct TemplateAsset;

pub fn get_template_asset_string(path: &PathBuf) -> Result<String, Box<Error>> {
    let path = path.to_str().unwrap().trim_start_matches("/");
    match TemplateAsset::get(path) {
        Some(content) => {
            let text: String = match content {
                Cow::Borrowed(bytes) => String::from_utf8(bytes.to_vec()).unwrap(),
                Cow::Owned(bytes) => String::from_utf8(bytes.to_vec()).unwrap(),
            };
            Ok(text)
        }
        None => return Err(Box::new(ToolError::MissingTemplateAssetPath(path.to_string()))),
    }
}

pub fn get_template_asset_bytes(path: &PathBuf) -> Result<Vec<u8>, Box<Error>> {
    let path = path.to_str().unwrap().trim_start_matches("/");
    match TemplateAsset::get(path) {
        Some(content) => {
            let bytes: Vec<u8> = match content {
                Cow::Borrowed(bytes) => bytes.to_vec(),
                Cow::Owned(bytes) => bytes.to_vec(),
            };
            Ok(bytes)
        }
        None => return Err(Box::new(ToolError::MissingTemplateAssetPath(path.to_string()))),
    }
}

impl ProjectData {
    pub fn new(demo_yml_path: Option<PathBuf>, embedded: bool) -> Result<ProjectData, Box<Error>> {
        if let Some(yml_path) = demo_yml_path {
            info!("plazma::ProjectData::new() using yml_path: {:?} and embedded '{:?}'", &yml_path, &embedded);

            let text: String = if embedded {
                get_template_asset_string(&yml_path)?
            } else {
                file_to_string(&yml_path)?
            };

            let p = yml_path.parent().ok_or("missing demo yml parent folder")?;
            let project_root = p.to_path_buf();

            let dmo_data = DmoData::new_from_yml_str(&text, &Some(project_root.clone()), true, true, embedded)?;

            // TODO optimize for the same shader being used at different scenes

            Ok(ProjectData {
                project_root: Some(project_root),
                demo_yml_path: Some(yml_path.clone()),
                dmo_data: dmo_data,
                embedded: embedded,
            })
        } else {
            info!("plazma::ProjectData::new() with DmoData::new_minimal()");
            Ok(ProjectData {
                project_root: None,
                demo_yml_path: None,
                dmo_data: DmoData::new_minimal()?,
                embedded: false,
            })
        }
    }

    pub fn new_from_embedded_template(template: NewProjectTemplate) -> Result<ProjectData, Box<Error>> {
        info!("ProjectData::new_from_template() {:?}", template);

        use NewProjectTemplate::*;
        let p = match template {
            QuadShader => "custom_quad/demo.yml",
            PolygonScene =>  "custom_polygon/demo.yml",
            ShadertoyDefault => "shadertoy_default/demo.yml",
            ShadertoyRaymarch =>  "shadertoy_raymarch/demo.yml",

            //ShadertoyTunnel => {},
            //ShadertoyVolumetric => {},
            //ShadertoyLattice => {},
            //ShadertoyFractal => {},
            //ShadertoyPbr => {},
            //BonzomaticTunnel => {},

            _ => "custom_quad/demo.yml",
        };
        ProjectData::new(Some(PathBuf::from(p)), true)
    }

    pub fn write_shaders(&self) -> Result<(), Box<Error>> {
        if let Some(ref project_root) = self.project_root {

            for (path, idx) in self.dmo_data.context.index.get_shader_path_to_idx().iter() {
                if path.starts_with("data_builtin_") {
                    continue;
                }

                let shader_path = project_root.join(PathBuf::from(&path));

                let mut file = match File::create(&shader_path) {
                    Ok(f) => f,
                    Err(e) => return Err(Box::new(e)),
                };

                match file.write_all(self.dmo_data.context.shader_sources[*idx].as_bytes()) {
                    Ok(_) => {},
                    Err(e) => return Err(Box::new(e)),
                }

                info!{"Wrote {} bytes to {:?}", self.dmo_data.context.shader_sources[*idx].len(), &shader_path};

            }
        } else {
            return Err(Box::new(ToolError::MissingProjectRoot));
        }

        Ok(())
    }
}

impl Default for ProjectData {
    fn default() -> ProjectData {
        ProjectData {
            project_root: None,
            demo_yml_path: None,
            dmo_data: DmoData::default(),
            embedded: false,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum NewProjectTemplate {
    QuadShader,
    PolygonScene,
    ShadertoyDefault,
    ShadertoyRaymarch,
    ShadertoyTunnel,
    ShadertoyVolumetric,
    ShadertoyLattice,
    ShadertoyFractal,
    ShadertoyPbr,
    BonzomaticTunnel,
}

