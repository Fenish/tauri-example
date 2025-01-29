use std::fs;
use std::path::Path;

/// Creates the directory if it doesn't exist.
pub fn create_dir_if_not_exists(path: &Path) -> Result<(), std::io::Error> {
    if !path.exists() {
        fs::create_dir_all(path)?;
        println!("Directory created at: {:?}", path);
    } else {
        println!("Directory already exists at: {:?}", path);
    }
    Ok(())
}

