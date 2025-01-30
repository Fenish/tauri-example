use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;

// Global mutable image cache directory
pub static IMAGE_CACHE_DIR: Lazy<Mutex<PathBuf>> = Lazy::new(|| Mutex::new(PathBuf::new()));
