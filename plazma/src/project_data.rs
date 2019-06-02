use std::path::PathBuf;
use std::error::Error;

use crate::utils::file_to_string;
use crate::dmo_data::DmoData;

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

