use tauri_plugin_dialog::DialogExt;
use tauri::AppHandle;
use tauri_plugin_dialog::FilePath;

pub fn open_image_dialog(app_handle: AppHandle) -> Vec<String> {
    let file_path: Option<FilePath> = app_handle
        .dialog()
        .file()
        .add_filter("Image Files", &["png", "jpeg", "jpg", "gif", "webp", "bmp", "tiff", "svg"])
        .blocking_pick_file();

    file_path.map(|path| vec![path.to_string()]).unwrap_or_default()
}
