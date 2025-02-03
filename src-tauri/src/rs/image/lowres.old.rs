use image::imageops::FilterType;
use image::GenericImageView;
use serde::Serialize;
use std::fs::{metadata, copy};
use std::path::Path;
use tauri::AppHandle;
use tauri::Emitter;
use exif::{Reader, Tag};
use std::fs::File;
use sha2::{Sha256, Digest};
use std::io::Read;

use crate::global::IMAGE_CACHE_DIR;
use crate::rs::utils::file_utils;

const TARGET_SIZE: u32 = 1024;

#[derive(Serialize)]
pub struct ImageSizes {
    pub lowres: String,
    pub hires: String,
}

#[derive(Serialize)]
pub struct ImageDimensions {
    pub lowres: String,
    pub hires: String,
}

#[derive(Serialize)]
pub struct ImageDetails {
    pub name: String,
    pub size: ImageSizes,
    pub dimensions: ImageDimensions,
    pub lowres_path: String,
    pub hires_path: String,
    pub dpi: Option<(f32, f32)>,
}

fn get_resolutions(image_path: &str) -> Option<(f32, f32)> {
    // Open the image file
    let file = File::open(image_path).ok()?;
    
    // Create an EXIF reader
    let reader = Reader::new();
    let buf = reader.read_from_container(&mut std::io::BufReader::new(file)).ok()?;

    let mut x_resolution = None;
    let mut y_resolution = None;

    // Look for XResolution and YResolution in the EXIF data
    for f in buf.fields() {
        match f.tag {
            Tag::XResolution => {
                if let exif::Value::Rational(r) = &f.value {
                    x_resolution = Some(r[0].to_f32());
                }
            },
            Tag::YResolution => {
                if let exif::Value::Rational(r) = &f.value {
                    y_resolution = Some(r[0].to_f32());
                }
            },
            _ => {}
        }
    }

    // If both resolutions are found, return them as a tuple
    match (x_resolution, y_resolution) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    }
}

fn get_human_readable_size(size_in_bytes: u64) -> String {
    let size_in_kb = size_in_bytes as f64 / 1024.0;
    let size_in_mb = size_in_kb / 1024.0;

    if size_in_mb >= 1.0 {
        format!("{:.2} MB", size_in_mb)
    } else {
        format!("{:.2} KB", size_in_kb)
    }
}

fn calculate_progress(image_index: usize, total_images: usize, step: u8, total_steps: u8) -> f64 {
    let image_progress = (step as f64 / total_steps as f64) * 100.0;
    let base_progress = (image_index as f64 / total_images as f64) * 100.0;
    let step_size = 100.0 / total_images as f64;
    
    base_progress + (step_size * (image_progress / 100.0))
}

fn calculate_new_dimensions(width: u32, height: u32) -> Option<(u32, u32)> {
    let longest_edge = width.max(height);
    
    // If the longest edge is already smaller than or equal to target size,
    // no resizing needed
    if longest_edge <= TARGET_SIZE {
        return None;
    }

    // Calculate the scaling factor to maintain aspect ratio
    let scale = TARGET_SIZE as f64 / longest_edge as f64;
    let new_width = (width as f64 * scale).round() as u32;
    let new_height = (height as f64 * scale).round() as u32;

    Some((new_width, new_height))
}

fn calculate_file_hash(file_path: &str) -> String {
    let mut hasher = Sha256::new();
    let mut file = File::open(file_path).expect("Unable to open file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Unable to read file");
    hasher.update(&buffer);
    format!("{:x}", hasher.finalize())
}

#[tauri::command]
pub async fn load_and_resize_images(app_handle: AppHandle) -> Result<Vec<ImageDetails>, String> {
    let image_cache_dir = IMAGE_CACHE_DIR.lock().unwrap().clone();
    let lowres_dir = image_cache_dir.join("lowres");
    let hires_dir = image_cache_dir.join("hires");

    // Open the file dialog to select multiple image files
    let selected_files = file_utils::open_image_dialog(app_handle.clone());

    if selected_files.is_empty() {
        return Ok(vec![]);
    }

    let mut image_details_list = Vec::new();
    let total_images = selected_files.len();
    const TOTAL_STEPS: u8 = 6; // Total number of steps per image

    // Initial progress
    app_handle.emit("image-loading-progress", 0.0).unwrap();

    for (index, selected_file) in selected_files.iter().enumerate() {
        // Step 1: Loading image
        app_handle.emit("image-loading-progress", 
            calculate_progress(index, total_images, 1, TOTAL_STEPS)).unwrap();
            
        let img = match image::open(&selected_file) {
            Ok(img) => img,
            Err(e) => return Err(format!("Failed to load image: {}", e)),
        };

        // Step 2: Processing dimensions
        app_handle.emit("image-loading-progress", 
            calculate_progress(index, total_images, 2, TOTAL_STEPS)).unwrap();
            
        let (width, height) = img.dimensions();
        let hires_dimensions = format!("{}x{}", width, height);

        let file_hash = calculate_file_hash(selected_file);
        let new_filename = format!("{}-{}.jpg", Path::new(&selected_file).file_stem().unwrap().to_string_lossy(), file_hash);
        let hires_path = hires_dir.join(&new_filename);

        if !hires_path.exists() {
            // Step 3: Copying original image (hires)
            app_handle.emit("image-loading-progress", 
                calculate_progress(index, total_images, 3, TOTAL_STEPS)).unwrap();
                
            if let Err(e) = copy(&selected_file, &hires_path) {
                return Err(format!("Failed to copy original image: {}", e));
            }
        }

        // Calculate new dimensions if needed
        let needs_resize = calculate_new_dimensions(width, height).is_some();
        
        // Step 4: Process lowres version if needed
        app_handle.emit("image-loading-progress", 
            calculate_progress(index, total_images, 4, TOTAL_STEPS)).unwrap();

        let (lowres_path, lowres_dimensions) = if needs_resize {
            if hires_path.exists() {
                (hires_path.clone(), hires_dimensions.clone())
            } else {
                let (new_width, new_height) = calculate_new_dimensions(width, height).unwrap();
                let lowres_filename = format!("{}-{}-lowres.jpg", Path::new(&selected_file).file_stem().unwrap().to_string_lossy(), file_hash);
                let lowres_path = lowres_dir.join(&lowres_filename);
                
                let low_res_image = img.resize(new_width, new_height, FilterType::Lanczos3);
                if let Err(e) = low_res_image.save(&lowres_path) {
                    return Err(format!("Failed to save low-res image: {}", e));
                }
                
                (lowres_path, format!("{}x{}", new_width, new_height))
            }
        } else {
            // If no resize needed, use the hires path for both
            (hires_path.clone(), hires_dimensions.clone())
        };

        // Step 5: Processing metadata
        app_handle.emit("image-loading-progress", 
            calculate_progress(index, total_images, 5, TOTAL_STEPS)).unwrap();
            
        let hires_size = match metadata(&hires_path) {
            Ok(meta) => get_human_readable_size(meta.len()),
            Err(_) => "Unknown size".to_string(),
        };

        let lowres_size = if needs_resize {
            match metadata(&lowres_path) {
                Ok(meta) => get_human_readable_size(meta.len()),
                Err(_) => "Unknown size".to_string(),
            }
        } else {
            hires_size.clone() // Use same size if no resize
        };

        let dpi_data = get_resolutions(selected_file);

        // Step 6: Finalizing
        app_handle.emit("image-loading-progress", 
            calculate_progress(index, total_images, 6, TOTAL_STEPS)).unwrap();

        image_details_list.push(ImageDetails {
            name: Path::new(&selected_file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            size: ImageSizes {
                lowres: lowres_size,
                hires: hires_size,
            },
            dimensions: ImageDimensions {
                lowres: lowres_dimensions,
                hires: hires_dimensions,
            },
            lowres_path: lowres_path.to_string_lossy().into_owned(),
            hires_path: hires_path.to_string_lossy().into_owned(),
            dpi: dpi_data,
        });
    }

    // Return the list of image details
    Ok(image_details_list)
}
