use image::codecs::png::PngEncoder;
use tauri::ipc::Channel;
use std::time::Instant;
use tauri::AppHandle;
use serde::Serialize;
use tokio::io::{AsyncReadExt, SeekFrom};
use sha2::{Sha256, Digest};
use tokio::io::AsyncSeekExt;

use std::path::{Path, PathBuf};
use tokio::fs::File;
use image::{ExtendedColorType, ImageEncoder, ImageReader};
use fast_image_resize::{IntoImageView, Resizer};
use fast_image_resize::images::Image;
use image::GenericImageView;
use std::io::BufWriter;
use tokio::io::AsyncWriteExt;

use crate::global::IMAGE_CACHE_DIR;
use crate::rs::utils::file_utils;


const TARGET_SIZE: u32 = 1024;


#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum DownloadEvent<'a> {
	#[serde(rename_all = "camelCase")]
	Started {
		file_name: &'a str,
		content_length: u64,
	},
	#[serde(rename_all = "camelCase")]
	Progress {
		file_name: &'a str,
		chunk: Vec<u8>,
	},
	#[serde(rename_all = "camelCase")]
	Finished {
		file_name: &'a str,
	},
}

#[tauri::command]
pub async fn load_and_resize_images(app_handle: AppHandle, channel: Channel<DownloadEvent<'_>>) -> Result<(), String> {
    let image_cache_dir = IMAGE_CACHE_DIR.lock().unwrap().clone();
    let lowres_dir: std::path::PathBuf = image_cache_dir.join("lowres");

    let start_time = Instant::now();
    let selected_files = file_utils::open_image_dialog(app_handle.clone());

    for file in selected_files {
        println!("Loading image: {}", file);

        // Open the image file and read it in chunks
		let mut lowres_image = get_lowres_image(&file);
        let mut file: File = File::open(file).await.unwrap();

		// Get hash of file
		let mut buffer = Vec::new();
        let mut hasher = Sha256::new();
        file.read_to_end(&mut buffer).await.unwrap();
		file.seek(SeekFrom::Start(0)).await.unwrap();
        hasher.update(&buffer);

        let file_hash = format!("{:x}", hasher.finalize());
		let lowres_path = lowres_dir.join(format!("{}.png", file_hash));

		// Save Lowres image to lowres_dir
		let mut lowres_file = File::create(&lowres_path).await.unwrap();
		let mut chunk = Vec::new();
		let mut len = file.read_to_end(&mut chunk).await.unwrap();
		while len > 0 {
			lowres_file.write_all(&chunk[..len]).await.unwrap();
			len = file.read_to_end(&mut chunk).await.unwrap();
		}
		println!("Saved lowres image to: {}", lowres_path.display());

		// let total_length = file.metadata().await.unwrap().len();

		// println!("Total length: {}", total_length);
		// channel.send(DownloadEvent::Started { file_name: &file_name, content_length: total_length }).unwrap();


		// let chunk_size = (total_length / 100).max(1) as usize;
		// let mut chunk = vec![0; chunk_size];
		// loop {
		// 	let len = file.read(&mut chunk).await.unwrap();
		// 	println!("Read {} bytes", len);
		// 	if len == 0 {
		// 		break;
		// 	}
		// 	channel.send(DownloadEvent::Progress { file_name: &file_name, chunk: chunk.to_vec() }).unwrap();
		// }
		// channel.send(DownloadEvent::Finished { file_name: &file_name }).unwrap();
    }

    let elapsed_time = start_time.elapsed();
    println!("Time taken to import images: {:?}", elapsed_time);
    Ok(())
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


fn get_lowres_image(path: &str) {
	let src_image = ImageReader::open(path)
		.unwrap()
		.decode()
		.unwrap();

	let (width, height) = src_image.dimensions();
	let (new_width, new_height) = calculate_new_dimensions(width, height).unwrap();

	let mut dst_image = Image::new(new_width, new_height, src_image.pixel_type().unwrap());
	let mut resizer = Resizer::new();
	resizer.resize(&src_image, &mut dst_image, None).unwrap();

	let mut result = BufWriter::new(Vec::new());
	PngEncoder::new(&mut result)
		.write_image(
			dst_image.buffer(),
			new_width,
			new_height,
			src_image.color().into(),
		)
		.unwrap();

}
