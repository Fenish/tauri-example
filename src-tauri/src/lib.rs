pub mod global;
pub mod image;
pub mod utilities;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            crate::image::lowres_rs::load_and_resize_images,
        ])
        .setup(|app| {
            // Initialize the cache directory once the app is running
            let app_config_dir = app.path().app_config_dir().unwrap();
            let image_cache_dir = app_config_dir.join("image_cache");

            utilities::file_utils::create_dir_if_not_exists(&image_cache_dir);
            utilities::file_utils::create_dir_if_not_exists(&image_cache_dir.join("highres"));
            utilities::file_utils::create_dir_if_not_exists(&image_cache_dir.join("lowres"));
            utilities::file_utils::create_dir_if_not_exists(&image_cache_dir.join("tiles"));

            *global::IMAGE_CACHE_DIR.lock().unwrap() = image_cache_dir;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
