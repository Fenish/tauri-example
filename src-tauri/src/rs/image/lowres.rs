use image::GenericImageView;
use native_dialog::FileDialog;
use std::path::Path;
use tauri::{self};
use tauri::Manager;


#[tauri::command]
pub async fn load_and_resize_images(app: tauri::AppHandle) -> Result<Vec<String>, String> {
	let _app_config_dir = app.path().app_config_dir().unwrap();

    // Open the file dialog to select an image file
    let selected_file = open_image_dialog().await;

    // Handle if no file was selected
    let selected_file = match selected_file {
        Some(file) => file,
        None => return Err("No file selected".to_string()),
    };

    // Load the image using the image crate
    let img = match image::open(&selected_file) {
        Ok(img) => img,
        Err(e) => return Err(format!("Failed to load image: {}", e)),
    };

    // Resize the image to a low resolution (e.g., 10% of original size)
    let scale_factor = 0.1;
    let (width, height) = img.dimensions();
    let new_width = (width as f64 * scale_factor) as u32;
    let new_height = (height as f64 * scale_factor) as u32;

    let low_res_image = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

    // Get the file name from the path
    let path = Path::new(&selected_file);
    let file_name = match path.file_name() {
        Some(name) => name.to_str().unwrap_or("unknown").to_string(),
        None => "unknown".to_string(),
    };

    // Save the low-resolution image
    let output_path = format!("low_res_{}", file_name);
    match low_res_image.save(&output_path) {
        Ok(_) => Ok(vec![output_path]),
        Err(e) => Err(format!("Failed to save low-res image: {}", e)),
    }
}

async fn open_image_dialog() -> Option<String> {
    // Open the file dialog using native_dialog to select a file
    match FileDialog::new()
        .add_filter("Image Files", &["png", "jpg", "jpeg", "bmp", "gif"])
        .show_open_single_file()
    {
        Ok(Some(file_path)) => Some(file_path.display().to_string()), // File selected
        Ok(None) => None, // No file selected
        Err(_) => None,   // Error occurred
    }
}
