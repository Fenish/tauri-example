use std::path::PathBuf;

use fimg::scale::Lanczos3;
use fimg::{DynImage, Image};
use image::ImageReader;
use serde_json::json;
use sha2::{Sha256, Digest};
use tauri::{ipc::Channel, AppHandle};
use tokio::time::Instant;
use rexiv2::Metadata;

use crate::global::IMAGE_CACHE_DIR;
use crate::utilities::file_utils;

const MAXIMUM_DIMENSION: u32 = 1024;

fn prepare_directories(image_cache_dir: &PathBuf) -> (PathBuf, PathBuf) {
    let lowres_dir = image_cache_dir.join("lowres");
    let highres_dir = image_cache_dir.join("highres");

    file_utils::create_dir_if_not_exists(&image_cache_dir);
    file_utils::create_dir_if_not_exists(&lowres_dir);
    file_utils::create_dir_if_not_exists(&highres_dir);

    (lowres_dir, highres_dir)
}

fn calculate_new_dimensions(image: &Image<Vec<u8>, 3>) -> Option<(u32, u32)> {
    let width = image.width() as f32;
    let height = image.height() as f32;

    let longest_edge = width.max(height);
    
    // If image is smaller than or equal to MAXIMUM_DIMENSION, return None
    if longest_edge <= MAXIMUM_DIMENSION as f32 {
        return None;
    }

    let scale_factor = longest_edge / MAXIMUM_DIMENSION as f32;

    let new_width = (width / scale_factor).round() as u32;
    let new_height = (height / scale_factor).round() as u32;

    Some((new_width, new_height))
}

fn get_image_hash(image: &Image<Vec<u8>, 3>) -> String {
	let mut hasher = Sha256::new();
	hasher.update(image.bytes());
	let hash = hasher.finalize();
	hex::encode(hash)
}

fn get_lowres_image(image: &mut Image<Vec<u8>, 3>, image_path: &PathBuf, new_width: u32, new_height: u32) -> Image<Box<[u8]>, 3> {
    let output;
    if image_path.exists() {
        output = DynImage::open(image_path).to_rgb();
    } else {
        // Create a new image with the correct dimensions and pixel data
        let scaled = image.scale::<Lanczos3>(new_width, new_height);
        
        // Save the RGB image directly
        scaled.save(image_path);
        
        output = scaled;
    }

    output
}

fn get_dpi(image_path: &PathBuf) -> Option<u32> {
    if let Ok(metadata) = Metadata::new_from_path(image_path) {
        let x_resolution = metadata.get_tag_string("Exif.Image.XResolution");
        let y_resolution = metadata.get_tag_string("Exif.Image.YResolution");
        
        if let (Ok(x_res), Ok(y_res)) = (x_resolution, y_resolution) {
            // Parse X resolution
            let x_resolution_value = x_res.split('/')
                .map(|x| x.parse::<f64>().unwrap_or(72.0))
                .collect::<Vec<f64>>();
            let x_dpi = if x_resolution_value.len() >= 2 && x_resolution_value[1] != 0.0 {
                x_resolution_value[0] / x_resolution_value[1]
            } else {
                72.0
            };

            // Parse Y resolution
            let y_resolution_value = y_res.split('/')
                .map(|x| x.parse::<f64>().unwrap_or(72.0))
                .collect::<Vec<f64>>();
            let y_dpi = if y_resolution_value.len() >= 2 && y_resolution_value[1] != 0.0 {
                y_resolution_value[0] / y_resolution_value[1]
            } else {
                72.0
            };

            // Use the average of X and Y DPI, or just X if they're the same
            Some(if (x_dpi - y_dpi).abs() < 0.1 {
                x_dpi as u32
            } else {
                ((x_dpi + y_dpi) / 2.0) as u32
            })
        } else {
            Some(72)
        }
    } else {
        Some(72) // Default to 72 DPI if metadata can't be read
    }
}

fn get_image_data(image_path: PathBuf) -> String {
    // Get file size
    let metadata = std::fs::metadata(&image_path).expect("Unable to read metadata");
    let file_size = metadata.len();
    let size_in_kb = file_size as f64 / 1024.0;
    let size_in_mb = size_in_kb / 1024.0;
    let size_in_gb = size_in_mb / 1024.0;

    if size_in_gb >= 1.0 {
        format!("{:.2} GB", size_in_gb)
    } else if size_in_mb >= 1.0 {
        format!("{:.2} MB", size_in_mb)
    } else {
        format!("{:.2} KB", size_in_kb)
    }
}

fn calculate_progress(current_step: usize, total_steps: usize) -> f32 {
    if total_steps == 0 {
        return 100.0;
    }

    let progress = (current_step as f32 / total_steps as f32) * 100.0;
    if progress > 100.0 {
        100.0
    } else {
        progress
    }
}

#[tauri::command]
pub async fn load_and_resize_images(app_handle: AppHandle, channel: Channel) -> Result<Vec<serde_json::Value>, ()> {
    let image_cache_dir = IMAGE_CACHE_DIR.lock().unwrap().clone();
    let (lowres_dir, highres_dir) = prepare_directories(&image_cache_dir);

    let selected_files = file_utils::open_image_dialog(app_handle);
    let start_time = Instant::now();
    
    let total_steps = selected_files.len() * 5; // 5 steps per file
    let mut current_step = 0;

	let mut results = Vec::new();
    // Loop selected_files and open image
    for file in &selected_files {
        // Step 1: Opening and decoding image
        current_step += 1;
        let _ = channel.send(tauri::ipc::InvokeResponseBody::Json(serde_json::json!({
            "event": "progress",
            "data": {
                "percentage": calculate_progress(current_step, total_steps),
                "step": "Opening image"
            }
        }).to_string()));

        let mut reader = ImageReader::open(file)
            .expect("Failed to open image")
            .with_guessed_format()
            .expect("Failed to guess image format");

        reader.no_limits();
        let source = reader.decode()
            .expect("Failed to decode image");

        // Step 2: Converting to RGB and generating hash
        current_step += 1;
        let _ = channel.send(tauri::ipc::InvokeResponseBody::Json(serde_json::json!({
            "event": "progress",
            "data": {
                "percentage": calculate_progress(current_step, total_steps),
                "step": "Processing image"
            }
        }).to_string()));

        let highres_image:Image<Vec<u8>, 3> = Image::<_, 3>::build(source.width(), source.height()).buf(source.into_rgb8().into_raw());
        let hash = get_image_hash(&highres_image);

        // Step 3: Creating low-res version
        current_step += 1;
        let _ = channel.send(tauri::ipc::InvokeResponseBody::Json(serde_json::json!({
            "event": "progress",
            "data": {
                "percentage": calculate_progress(current_step, total_steps),
                "step": "Creating low-res version"
            }
        }).to_string()));

        let lowres_destination = lowres_dir.join(hash.clone() + ".png");
        let highres_destination = highres_dir.join(hash.clone() + ".tiff");

        let highres_width = highres_image.width();
        let highres_height = highres_image.height();
        
        // Calculate dimensions and decide if we need a lowres version
        let lowres_info = calculate_new_dimensions(&highres_image);
        
        let (lowres_path, lowres_width, lowres_height) = if let Some((width, height)) = lowres_info {
            // Create lowres version only if needed
            get_lowres_image(&mut highres_image.clone(), &lowres_destination, width, height);
            (lowres_destination.to_str().unwrap(), width, height)
        } else {
            // Use highres path for both if image is small enough
            (highres_destination.to_str().unwrap(), highres_width, highres_height)
        };

        // Step 4: Saving high-res version
        current_step += 1;
        let _ = channel.send(tauri::ipc::InvokeResponseBody::Json(serde_json::json!({
            "event": "progress",
            "data": {
                "percentage": calculate_progress(current_step, total_steps),
                "step": "Saving high-res version"
            }
        }).to_string()));

        if !highres_destination.exists() {
            // Save as RGB for now
            highres_image.save(&highres_destination);
        }

        // Step 5: Getting image metadata
        current_step += 1;
        let _ = channel.send(tauri::ipc::InvokeResponseBody::Json(serde_json::json!({
            "event": "progress",
            "data": {
                "percentage": calculate_progress(current_step, total_steps),
                "step": "Extracting metadata"
            }
        }).to_string()));

        let highres_size_str = get_image_data(highres_destination.clone());
        let lowres_size_str = if lowres_info.is_some() {
            get_image_data(lowres_destination.clone())
        } else {
            highres_size_str.clone()
        };
        let dpi = get_dpi(&highres_destination);

        let filename = file.split("/").last().unwrap();

        let output = json!({
            "filename": filename,
            "dpi": dpi,
            "paths": {
                "highres": highres_destination.to_str().unwrap(),
                "lowres": lowres_path
            },
            "sizes": {
                "highres": highres_size_str,
                "lowres": lowres_size_str
            },
            "dimensions": {
                "highres": {
                    "width": highres_width,
                    "height": highres_height
                },
                "lowres": {
                    "width": lowres_width,
                    "height": lowres_height
                }
            }
        });

        results.push(output);
    }

    let end_time = Instant::now();
    let time_taken = end_time.duration_since(start_time);
    
    // Send completion message
    let _ = channel.send(tauri::ipc::InvokeResponseBody::Json(serde_json::json!({
        "event": "complete",
        "data": {
            "time_taken": format!("{:.2?}", time_taken),
            "total_files": selected_files.len()
        }
    }).to_string()));

	Ok(results)
}
