use std::path::PathBuf;

use tauri_plugin_dialog::DialogExt;
use tauri::AppHandle;
use tauri_plugin_dialog::FilePath;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub fn open_image_dialog(app_handle: AppHandle) -> Vec<String> {
    let file_paths: Option<Vec<FilePath>> = app_handle
        .dialog()
        .file()
        .add_filter("Image Files", &["png", "jpeg", "jpg", "gif", "webp", "bmp", "tiff", "svg"])
        .blocking_pick_files();

    file_paths.map(|paths| paths.into_iter().map(|path| path.to_string()).collect()).unwrap_or_default()
}


pub async fn save_file(buffer: Vec<u8>, output_path: &PathBuf) -> Result<(), String> {
    // Create or open the file where you want to save the image asynchronously
    let mut file = File::create(output_path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    // Write the byte data into the file asynchronously
    file.write_all(&buffer)
        .await
        .map_err(|e| format!("Failed to write data to file: {}", e))?;

    Ok(())
}

pub async fn check_file_exists(file_path: &PathBuf) -> bool {
	File::open(file_path).await.is_ok()
}