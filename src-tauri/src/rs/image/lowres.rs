use tauri::{ipc::Channel, AppHandle};

use tokio::{fs::File, io::AsyncReadExt, io::AsyncSeekExt};
use image::GenericImageView;

use std::io::{BufWriter, SeekFrom};
use sha2::{Sha256, Digest};

use image::codecs::png::PngEncoder;
use image::{ImageEncoder, ImageReader};

use fast_image_resize::{IntoImageView, Resizer};
use fast_image_resize::images::Image;
use fast_image_resize as fr;

use crate::global::IMAGE_CACHE_DIR;
use crate::rs::utils::file_utils;

use serde::{Serialize, Deserialize};
use std::fs::metadata;


#[derive(Serialize, Deserialize, Debug)]
pub struct ImageInfo {
    pub image_name: String,
    pub size: ImageSize,
    pub dimensions: ImageDimensions,
    pub paths: ImagePaths,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageSize {
    pub lowres: String,
    pub highres: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageDimensions {
    pub lowres: XYDimensions,
    pub highres: XYDimensions,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct XYDimensions {
    pub x: u32,
    pub y: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImagePaths {
    pub lowres: String,
    pub highres: String,
}

pub async fn get_image_info(image_name: &str, file_path: &str) -> Result<ImageInfo, String> {
    // Load high-res image
    let highres_image = ImageReader::open(file_path)
        .map_err(|e| format!("Failed to open highres image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode highres image: {}", e))?;
    let (highres_width, highres_height) = highres_image.dimensions();

    // Get high-res file size
    let highres_size = get_file_size(file_path)?;

    // Calculate low-res path and load low-res image
    let image_cache_dir = IMAGE_CACHE_DIR.lock().unwrap().clone();
    let lowres_dir: std::path::PathBuf = image_cache_dir.join("lowres");
    let lowres_path = lowres_dir.join(format!("{}.png", image_name));
    let lowres_image = ImageReader::open(&lowres_path)
        .map_err(|e| format!("Failed to open lowres image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode lowres image: {}", e))?;
    let (lowres_width, lowres_height) = lowres_image.dimensions();

    // Get low-res file size
    let lowres_size = get_file_size(&lowres_path.to_string_lossy())?;

    // Build and return the ImageInfo struct
    let size = ImageSize {
        lowres: lowres_size,
        highres: highres_size,
    };

    let dimensions = ImageDimensions {
        lowres: XYDimensions {
			x: lowres_width,
			y: lowres_height
		},
        highres: XYDimensions {
			x: highres_width,
			y: highres_height
		},
    };

    let paths = ImagePaths {
        lowres: lowres_path.to_string_lossy().into(),
        highres: file_path.to_string(),
    };

    Ok(ImageInfo {
        image_name: image_name.to_string(),
        size,
        dimensions,
        paths,
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


#[tauri::command]
pub async fn load_and_resize_images(app_handle: AppHandle, channel: Channel) -> Result<Vec<ImageInfo>, String> {
    let image_cache_dir = IMAGE_CACHE_DIR.lock().unwrap().clone();
    let lowres_dir: std::path::PathBuf = image_cache_dir.join("lowres");
    let highres_dir: std::path::PathBuf = image_cache_dir.join("highres");

    let selected_files = file_utils::open_image_dialog(app_handle.clone());
    
    let mut image_info_list: Vec<ImageInfo> = Vec::new();

    for file_path in selected_files {
        // Get image info for the file
        let file_name = get_hash(&file_path).await;
        let lowres_path = lowres_dir.join(format!("{}.png", file_name));

        let is_exists: bool = file_utils::check_file_exists(&lowres_path).await;
        // If lowres image does not exist, generate it
        if !is_exists {
            let lowres_image_buffer = get_lowres_image(&file_path.clone()).unwrap();
            file_utils::save_file(lowres_image_buffer, &lowres_path).await.unwrap();
        }

        // Get image info for both high-res and low-res images
        let lowres_image_info = get_image_info(&file_name, &lowres_path.to_string_lossy()).await?;
        let highres_image_info = get_image_info(&file_name, &file_path).await?;

		let real_highres_path = highres_dir.join(format!("{}.tiff", file_name));
        // Add image info to the list
        image_info_list.push(ImageInfo {
            image_name: lowres_image_info.image_name,  // We can just use the lowres image name for both
            size: ImageSize {
                lowres: lowres_image_info.size.lowres,
                highres: highres_image_info.size.highres,
            },
            dimensions: ImageDimensions {
                lowres: lowres_image_info.dimensions.lowres,
                highres: highres_image_info.dimensions.highres,
            },
            paths: ImagePaths {
                lowres: lowres_image_info.paths.lowres,
                highres: real_highres_path.to_string_lossy().into(),
            },
        });
    }

    Ok(image_info_list)
}

fn get_lowres_image(path: &str) -> Result<Vec<u8>, String> {
	let src_image = ImageReader::open(path)
        .map_err(|e| format!("Failed to open image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    // Get the source image's dimensions
    let (src_width, src_height) = src_image.dimensions();

    // Define the max dimension (1024px)
    let max_dim = 1024;
    let (dst_width, dst_height) = if src_width > src_height {
        let new_width = max_dim;
        let new_height = (src_height as f32 * (max_dim as f32 / src_width as f32)) as u32;
        (new_width, new_height)
    } else {
        let new_height = max_dim;
        let new_width = (src_width as f32 * (max_dim as f32 / src_height as f32)) as u32;
        (new_width, new_height)
    };

    // Create a new image with the resized dimensions
    let mut dst_image = Image::new(dst_width, dst_height, src_image.pixel_type().unwrap());

    // Create Resizer instance and resize the source image
    let mut resizer = Resizer::new();
    unsafe {
        resizer.set_cpu_extensions(fr::CpuExtensions::Sse4_1);
    }

    // Perform the resize operation
    resizer
        .resize(&src_image, &mut dst_image, None)
        .map_err(|e| format!("Failed to resize image: {}", e))?;

    // Create a buffer to hold the resulting PNG image
    let mut result_buf = BufWriter::new(Vec::new());

    // Encode the resized image as PNG and write it into the buffer
    PngEncoder::new(&mut result_buf)
        .write_image(
            dst_image.buffer(),
            dst_width,
            dst_height,
            src_image.color().into(),
        )
        .map_err(|e| format!("Failed to write PNG image: {}", e))?;

	Ok(result_buf.into_inner().map_err(|e| format!("Failed to get result buffer: {}", e))?)
}

async fn get_hash(file_path: &str) -> String {
	let mut file: File = File::open(file_path).await.unwrap();
	let mut buffer = Vec::new();
	let mut hasher = Sha256::new();
	file.seek(SeekFrom::Start(0)).await.unwrap();
	file.read_to_end(&mut buffer).await.unwrap();
	hasher.update(&buffer);

	format!("{:x}", hasher.finalize())
}