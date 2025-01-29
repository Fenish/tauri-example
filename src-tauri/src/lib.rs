pub mod cpp;
pub mod rs;
pub mod utils;

use std::sync::Mutex;
use std::path::PathBuf;
use lazy_static::lazy_static;
use tauri::Manager;

lazy_static! {
    static ref CACHE_DIR: Mutex<Option<PathBuf>> = Mutex::new(None); // Global cache dir path
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
		.invoke_handler(tauri::generate_handler![
			crate::rs::image::lowres::load_and_resize_images,
		])
		.setup(|app| {
            // Initialize the cache directory once the app is running
            let app_config_dir = app.path().app_config_dir().unwrap();
			let image_cache_dir = app_config_dir.join("image_cache");

			if let Err(e) = utils::create_dir_if_not_exists(&image_cache_dir) {
				eprintln!("Failed to create directory: {}", e);
			}

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
