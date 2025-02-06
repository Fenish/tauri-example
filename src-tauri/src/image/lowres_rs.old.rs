use tauri::{ipc::Channel, AppHandle};

use tokio::{fs::File, io::AsyncReadExt, time::Instant};

use std::io::{BufWriter, BufReader, Seek, SeekFrom};
use sha2::{Sha256, Digest};

use image::codecs::png::PngEncoder;
use image::codecs::tiff::TiffEncoder;
use image::{ImageEncoder, ImageReader, DynamicImage, ColorType, ExtendedColorType, ImageFormat};

use fast_image_resize::{IntoImageView, Resizer};
use fast_image_resize::images::Image;
use fast_image_resize as fr;

use crate::global::IMAGE_CACHE_DIR;
use crate::utilities::file_utils;

use serde::{Serialize, Deserialize};
use std::fs::{metadata, create_dir_all};
use std::path::PathBuf;
use futures::future::join_all;
use std::fs::File as StdFile;

const BUFFER_SIZE: usize = 65536; // 64KB buffer
const MAX_DIMENSION: u32 = 1024;
const HASH_BUFFER_SIZE: usize = 8192; // 8KB for hashing
const MAX_IMAGE_DIMENSION: u32 = 50000; // Maximum dimension we'll try to process

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageInfo {
    pub image_name: String,
    pub size: ImageSize,
    pub dimensions: ImageDimensions,
    pub paths: ImagePaths,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageSize {
    pub lowres: String,
    pub highres: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageDimensions {
    pub lowres: XYDimensions,
    pub highres: XYDimensions,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XYDimensions {
    pub x: u32,
    pub y: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImagePaths {
    pub lowres: String,
    pub highres: String,
}

fn get_dimensions_without_loading(path: &str) -> Result<(u32, u32), String> {
    let file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open file for dimensions: {}", e))?;
    let reader = BufReader::with_capacity(BUFFER_SIZE, file);
    ImageReader::new(reader)
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess format: {}", e))?
        .into_dimensions()
        .map_err(|e| format!("Failed to get dimensions: {}", e))
}

fn calculate_dimensions(src_width: u32, src_height: u32) -> (u32, u32) {
    if src_width > src_height {
        let new_width = MAX_DIMENSION;
        let new_height = (src_height as f32 * (MAX_DIMENSION as f32 / src_width as f32)) as u32;
        (new_width, new_height)
    } else {
        let new_height = MAX_DIMENSION;
        let new_width = (src_width as f32 * (MAX_DIMENSION as f32 / src_height as f32)) as u32;
        (new_width, new_height)
    }
}

pub async fn get_image_info(image_name: &str, file_path: &str) -> Result<ImageInfo, String> {
    // Get dimensions without loading the entire image
    let (highres_width, highres_height) = get_dimensions_without_loading(file_path)?;

    // Get high-res file size
    let highres_size = get_file_size(file_path)?;

    // Calculate low-res path and get low-res dimensions
    let image_cache_dir = IMAGE_CACHE_DIR.lock().unwrap().clone();
    let lowres_dir: std::path::PathBuf = image_cache_dir.join("lowres");
    let lowres_path = lowres_dir.join(format!("{}.png", image_name));
    let (lowres_width, lowres_height) = get_dimensions_without_loading(&lowres_path.to_string_lossy())?;

    // Get low-res file size
    let lowres_size = get_file_size(&lowres_path.to_string_lossy())?;

    Ok(ImageInfo {
        image_name: image_name.to_string(),
        size: ImageSize {
            lowres: lowres_size,
            highres: highres_size,
        },
        dimensions: ImageDimensions {
            lowres: XYDimensions {
                x: lowres_width,
                y: lowres_height
            },
            highres: XYDimensions {
                x: highres_width,
                y: highres_height
            },
        },
        paths: ImagePaths {
            lowres: lowres_path.to_string_lossy().into(),
            highres: file_path.to_string(),
        },
    })
}

// Helper function to get file size in appropriate units
fn get_file_size(file_path: &str) -> Result<String, String> {
    let metadata = metadata(file_path).map_err(|e| format!("Failed to get file metadata: {}", e))?;
    let size = metadata.len();
    let size_in_mb = size as f64 / (1024.0 * 1024.0); // Convert to MB
    let size_in_kb = size as f64 / 1024.0; // Convert to KB

    if size_in_mb > 1.0 {
        Ok(format!("{:.2} MB", size_in_mb))
    } else if size_in_kb > 1.0 {
        Ok(format!("{:.2} KB", size_in_kb))
    } else {
        Ok(format!("{} bytes", size))
    }
}

fn process_image_in_chunks(
    src_image: DynamicImage,
    dst_width: u32,
    dst_height: u32,
) -> Result<Vec<u8>, String> {
    // Create output image with smaller dimensions
    let mut dst_image = Image::new(
        dst_width,
        dst_height,
        src_image.pixel_type().ok_or("Invalid pixel type")?,
    );

    // Create and configure resizer with optimal settings
    let mut resizer = Resizer::new();
    unsafe {
        resizer.set_cpu_extensions(fr::CpuExtensions::Sse4_1);
    }

    // Perform the resize operation
    resizer
        .resize(&src_image, &mut dst_image, None)
        .map_err(|e| format!("Failed to resize image: {}", e))?;

    // Create a buffer with a pre-allocated capacity
    let buffer_capacity = (dst_width * dst_height * 4) as usize;
    let mut result_buf = BufWriter::with_capacity(
        buffer_capacity,
        Vec::with_capacity(buffer_capacity)
    );

    // Encode as PNG with optimal settings for size/quality
    PngEncoder::new_with_quality(
        &mut result_buf,
        image::codecs::png::CompressionType::Best,
        image::codecs::png::FilterType::Adaptive
    )
    .write_image(
        dst_image.buffer(),
        dst_width,
        dst_height,
        src_image.color().into(),
    )
    .map_err(|e| format!("Failed to write PNG image: {}", e))?;

    result_buf.into_inner().map_err(|e| format!("Failed to get result buffer: {}", e))
}

fn copy_to_highres(src_path: &str, dst_path: &PathBuf) -> Result<(), String> {
    // Open and read the source image
    let file = std::fs::File::open(src_path)
        .map_err(|e| format!("Failed to open source image: {}", e))?;
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    
    // First check dimensions
    let dimensions = ImageReader::new(&mut reader)
        .with_guessed_format()
        .map_err(|e| format!("Failed to create reader: {}", e))?
        .into_dimensions()
        .map_err(|e| format!("Failed to get dimensions: {}", e))?;

    if dimensions.0 > MAX_IMAGE_DIMENSION || dimensions.1 > MAX_IMAGE_DIMENSION {
        return Err(format!("Image dimensions too large: {}x{}", dimensions.0, dimensions.1));
    }

    // Reset reader position
    reader.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to reset reader: {}", e))?;

    // Create output file and directory
    if let Some(parent) = dst_path.parent() {
        create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let output_file = StdFile::create(dst_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = BufWriter::with_capacity(BUFFER_SIZE, output_file);

    // Try to determine format
    let format = ImageReader::new(&mut reader)
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess format: {}", e))?
        .format();

    // Reset reader again
    reader.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to reset reader: {}", e))?;

    // Process based on format
    match format {
        Some(ImageFormat::Tiff) => {
            // If it's already TIFF, just copy
            std::io::copy(&mut reader, &mut writer)
                .map_err(|e| format!("Failed to copy TIFF file: {}", e))?;
            Ok(())
        },
        _ => {
            // For other formats, decode and convert to TIFF
            let image = ImageReader::new(reader)
                .with_guessed_format()
                .map_err(|e| format!("Failed to create reader: {}", e))?
                .decode()
                .map_err(|e| format!("Failed to decode image while converting to tiff: {}", e))?;

            save_as_tiff(&image, dst_path)
        }
    }
}

fn get_lowres_image(path: &str) -> Result<Vec<u8>, String> {
    // Get dimensions first
    let file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open image: {}", e))?;
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    
    let dimensions = ImageReader::new(&mut reader)
        .with_guessed_format()
        .map_err(|e| format!("Failed to create reader: {}", e))?
        .into_dimensions()
        .map_err(|e| format!("Failed to get dimensions: {}", e))?;

    if dimensions.0 > MAX_IMAGE_DIMENSION || dimensions.1 > MAX_IMAGE_DIMENSION {
        return Err(format!("Image dimensions too large: {}x{}", dimensions.0, dimensions.1));
    }

    let (dst_width, dst_height) = calculate_dimensions(dimensions.0, dimensions.1);

    // Reset reader position
    reader.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to reset reader: {}", e))?;
    
    let src_image = ImageReader::new(reader)
        .with_guessed_format()
        .map_err(|e| format!("Failed to create format reader: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    process_image_in_chunks(src_image, dst_width, dst_height)
}

fn save_as_tiff(image: &DynamicImage, path: &PathBuf) -> Result<(), String> {
    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let file = StdFile::create(path)
        .map_err(|e| format!("Failed to create TIFF file: {}", e))?;
    let mut writer = BufWriter::new(file);

    // Convert ColorType to ExtendedColorType
    let color_type = match image.color() {
        ColorType::L8 => ExtendedColorType::L8,
        ColorType::La8 => ExtendedColorType::La8,
        ColorType::Rgb8 => ExtendedColorType::Rgb8,
        ColorType::Rgba8 => ExtendedColorType::Rgba8,
        ColorType::L16 => ExtendedColorType::L16,
        ColorType::La16 => ExtendedColorType::La16,
        ColorType::Rgb16 => ExtendedColorType::Rgb16,
        ColorType::Rgba16 => ExtendedColorType::Rgba16,
        ColorType::Rgb32F => ExtendedColorType::Rgb32F,
        ColorType::Rgba32F => ExtendedColorType::Rgba32F,
        _ => return Err("Unsupported color type for TIFF".to_string()),
    };

    TiffEncoder::new(&mut writer)
        .write_image(
            image.as_bytes(),
            image.width(),
            image.height(),
            color_type,
        )
        .map_err(|e| format!("Failed to encode TIFF: {}", e))?;

    Ok(())
}

async fn process_single_image(
    file_path: &str,
    lowres_dir: &PathBuf,
    highres_dir: &PathBuf,
) -> Result<ImageInfo, String> {
    let file_name = get_hash(file_path).await;
    let lowres_path = lowres_dir.join(format!("{}.png", file_name));
    let real_highres_path = highres_dir.join(format!("{}.tiff", file_name));

    // Generate lowres image if it doesn't exist
    if !file_utils::check_file_exists(&lowres_path).await {
        let lowres_image_buffer = get_lowres_image(file_path)?;
        file_utils::save_file(lowres_image_buffer, &lowres_path).await
            .map_err(|e| format!("Failed to save lowres image: {}", e))?;
    }

    // Copy to highres directory as TIFF if it doesn't exist
    if !file_utils::check_file_exists(&real_highres_path).await {
        copy_to_highres(file_path, &real_highres_path)?;
    }

    // Get image info
    let lowres_info = get_image_info(&file_name, &lowres_path.to_string_lossy()).await?;
    let highres_info = get_image_info(&file_name, &real_highres_path.to_string_lossy()).await?;

    Ok(ImageInfo {
        image_name: lowres_info.image_name,
        size: ImageSize {
            lowres: lowres_info.size.lowres,
            highres: highres_info.size.highres,
        },
        dimensions: ImageDimensions {
            lowres: lowres_info.dimensions.lowres,
            highres: highres_info.dimensions.highres,
        },
        paths: ImagePaths {
            lowres: lowres_info.paths.lowres,
            highres: real_highres_path.to_string_lossy().into(),
        },
    })
}

#[tauri::command]
pub async fn load_and_resize_images(app_handle: AppHandle, channel: Channel) -> Result<Vec<ImageInfo>, String> {
    let image_cache_dir = IMAGE_CACHE_DIR.lock().unwrap().clone();
    let lowres_dir: PathBuf = image_cache_dir.join("lowres");
    let highres_dir: PathBuf = image_cache_dir.join("highres");

    // Ensure directories exist
    create_dir_all(&lowres_dir)
        .map_err(|e| format!("Failed to create lowres directory: {}", e))?;
    create_dir_all(&highres_dir)
        .map_err(|e| format!("Failed to create highres directory: {}", e))?;

    let selected_files = file_utils::open_image_dialog(app_handle);
    let start_time = Instant::now();
    
    // Process images concurrently using tokio
    let futures: Vec<_> = selected_files.iter()
        .map(|file_path| process_single_image(file_path, &lowres_dir, &highres_dir))
        .collect();
    
    let results = join_all(futures).await;
    let image_info_list: Result<Vec<_>, String> = results.into_iter().collect();
    let image_info_list = image_info_list?;

    let end_time = Instant::now();
    let time_taken = end_time.duration_since(start_time);
    let time_taken_str = format!("{:.2}s", time_taken.as_secs_f64());
    
    let _ = channel.send(tauri::ipc::InvokeResponseBody::Json(serde_json::json!({
        "event": "load_complete",
        "data": {
            "time_taken": time_taken_str
        }
    }).to_string()));

    Ok(image_info_list)
}

async fn get_hash(file_path: &str) -> String {
    let file = File::open(file_path).await.unwrap();
    let mut reader = tokio::io::BufReader::with_capacity(HASH_BUFFER_SIZE, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; HASH_BUFFER_SIZE];

    loop {
        let n = reader.read(&mut buffer).await.unwrap();
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }

    format!("{:x}", hasher.finalize())
}