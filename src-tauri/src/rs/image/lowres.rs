use image::imageops::FilterType;
use image::GenericImageView;
use std::fs::metadata;
use std::path::Path;
use uuid::Uuid;
use serde::Serialize;

use crate::global::IMAGE_CACHE_DIR;
use crate::rs::image::utils;

#[derive(Serialize)]
pub struct ImageDetails {
    pub name: String,
    pub size: String,     // File size in human-readable format (KB/MB)
    pub dimensions: String, // Image dimensions (e.g., "800x600")
    pub buffer: Vec<u16>,    // Image data as a Vec<u16> (16-bit)
}

#[tauri::command]
pub async fn load_and_resize_images() -> Result<Vec<ImageDetails>, String> {
    let image_cache_dir = IMAGE_CACHE_DIR.lock().unwrap().clone();

    // Open the file dialog to select multiple image files
    let selected_files = utils::open_image_dialog().await;

    if selected_files.is_empty() {
        return Ok(vec![]);
    }

    let mut image_details_list = Vec::new();

    for selected_file in selected_files {
        // Load the image using the image crate
        let img = match image::open(&selected_file) {
            Ok(img) => img,
            Err(e) => return Err(format!("Failed to load image: {}", e)),
        };

        // Resize the image to a low resolution (e.g., 50% of original size)
        let scale_factor = 0.5;
        let (width, height) = img.dimensions();
        let new_width = (width as f64 * scale_factor) as u32;
        let new_height = (height as f64 * scale_factor) as u32;

        let low_res_image = img.resize(new_width, new_height, FilterType::Lanczos3);

        // Extract original file extension (default to "png" if unknown)
        let original_extension = Path::new(&selected_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("png");

        // Generate a random file name using UUID and keep the original extension
        let random_file_name = format!("low_res_{}.{}", Uuid::new_v4(), original_extension);

        // Create full output path inside image_cache_dir
        let output_path = image_cache_dir.join(&random_file_name);

        // Save the low-resolution image
        if let Err(e) = low_res_image.save(&output_path) {
            return Err(format!("Failed to save low-res image: {}", e));
        }

        // Convert the image to 16-bit buffer
        let pixels: Vec<u16> = low_res_image.to_rgb16().into_raw();

        // Get the file size in human-readable format (KB/MB)
        let file_size = match metadata(&selected_file) {
            Ok(meta) => {
                let size_in_bytes = meta.len();
                let size_in_kb = size_in_bytes as f64 / 1024.0;
                let size_in_mb = size_in_kb / 1024.0;

                if size_in_mb >= 1.0 {
                    format!("{:.2} MB", size_in_mb)
                } else {
                    format!("{:.2} KB", size_in_kb)
                }
            }
            Err(_) => "Unknown size".to_string(),
        };

        // Get the image dimensions as a string (e.g., "800x600")
        let dimensions = format!("{}x{}", new_width, new_height);

        // Store the details of the resized image
        image_details_list.push(ImageDetails {
            name: selected_file,
            size: file_size,
            dimensions,
            buffer: pixels,
        });
    }

    // Return the list of image details
    Ok(image_details_list)
}
