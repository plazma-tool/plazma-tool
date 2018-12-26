use std::path::PathBuf;

use crate::dmo_data::DmoData;

pub struct ProjectData {
    pub project_path: PathBuf,
    pub project_root: PathBuf,
    pub dmo: DmoData,
}

impl Default for ProjectData {
    fn default() -> ProjectData {
        ProjectData {
            project_path: PathBuf::from(""),
            project_root: PathBuf::from(""),
            dmo: DmoData::default(),
        }
    }
}

