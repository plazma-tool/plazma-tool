use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::error::Error;

use crate::utils::file_to_string;
use crate::dmo_data::DmoData;
use crate::error::ToolError;

pub struct ProjectData {
    pub project_root: Option<PathBuf>,
    pub demo_yml_path: Option<PathBuf>,
    pub dmo_data: DmoData,
}

impl ProjectData {
    pub fn new(demo_yml_path: Option<PathBuf>) -> Result<ProjectData, Box<Error>> {
        if let Some(yml_path) = demo_yml_path {
            info!("plazma::ProjectData::new() using yml_path: {:?}", &yml_path);
            let text: String = file_to_string(&yml_path)?;

            let p = yml_path.parent().ok_or("missing demo yml parent folder")?;
            let project_root = p.to_path_buf();

            let dmo_data = DmoData::new_from_yml_str(&text, &Some(project_root.clone()), true, true)?;

            // TODO optimize for the same shader being used at different scenes

            Ok(ProjectData {
                project_root: Some(project_root),
                demo_yml_path: Some(yml_path.clone()),
                dmo_data: dmo_data,
            })
        } else {
            info!("plazma::ProjectData::new() with DmoData::new_minimal()");
            Ok(ProjectData {
                project_root: None,
                demo_yml_path: None,
                dmo_data: DmoData::new_minimal()?,
            })
        }
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
        }
    }
}

