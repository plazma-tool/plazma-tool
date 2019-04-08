use std::path::PathBuf;
use std::error::Error;

use serde_yaml::{self, Value};

use crate::utils::file_to_string;
use crate::dmo_data::DmoData;

pub struct ProjectData {
    pub project_root: PathBuf,
    pub demo_yml_path: PathBuf,
    pub demo_yml_value: Value,
    pub dmo_data: DmoData,
}

impl ProjectData {
    pub fn new(demo_yml_path: &PathBuf) -> Result<ProjectData, Box<Error>> {
        info!("ProjectData::new() demo_yml_path: {:?}", &demo_yml_path);
        let text: String = file_to_string(demo_yml_path)?;
        let demo_yml_value: Value = serde_yaml::from_str(&text)?;

        let project_root = demo_yml_path.parent().ok_or("missing demo yml parent")?;

        let dmo_data = DmoData::new_from_yml_str(&text, true, true)?;

        // TODO optimize for the same shader being used at different scenes

        Ok(ProjectData {
            project_root: project_root.to_path_buf(),
            demo_yml_path: demo_yml_path.clone(),
            demo_yml_value: demo_yml_value,
            dmo_data: dmo_data,
        })
    }
}

impl Default for ProjectData {
    fn default() -> ProjectData {
        ProjectData {
            project_root: PathBuf::default(),
            demo_yml_path: PathBuf::default(),
            demo_yml_value: Value::Null,
            dmo_data: DmoData::default(),
        }
    }
}

