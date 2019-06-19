use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// Takes a path to a file and try to read the file into a String
pub fn file_to_string(path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            error!("[file_to_string]: Failed to open {:?}", path);
            return Err(Box::new(e));
        }
    };

    let mut content = String::new();

    if let Err(e) = file.read_to_string(&mut content) {
        error!("[file_to_string]: Failed to read {:?}", path);
        return Err(Box::new(e));
    }

    Ok(content)
}

pub fn clean_windows_str_path(p: &str) -> &str {
    p.trim_start_matches("\\\\?\\")
}

