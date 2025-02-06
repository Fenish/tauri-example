use std::path::PathBuf;

use tauri::{ipc::Channel, AppHandle};
use tokio::time::Instant;

use crate::global::IMAGE_CACHE_DIR;
use crate::utilities::file_utils;

fn prepare_directories(image_cache_dir: &PathBuf) -> (PathBuf, PathBuf, PathBuf) {
    let lowres_dir = image_cache_dir.join("lowres");
    let highres_dir = image_cache_dir.join("highres");
    let tile_cache_dir = image_cache_dir.join("tiles");

    file_utils::create_dir_if_not_exists(&image_cache_dir);
    file_utils::create_dir_if_not_exists(&lowres_dir);
    file_utils::create_dir_if_not_exists(&highres_dir);
    file_utils::create_dir_if_not_exists(&tile_cache_dir);

    (lowres_dir, highres_dir, tile_cache_dir)
}

#[tauri::command]
pub async fn load_and_resize_images(app_handle: AppHandle, channel: Channel) {
    let image_cache_dir = IMAGE_CACHE_DIR.lock().unwrap().clone();
    let (lowres_dir, highres_dir, tile_cache_dir) = prepare_directories(&image_cache_dir);

    let selected_files = file_utils::open_image_dialog(app_handle);
    let start_time = Instant::now();
}
