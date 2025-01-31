use native_dialog::FileDialog;

pub async fn open_image_dialog() -> Vec<String> {
    // Open the file dialog for selecting multiple files
    match FileDialog::new()
        .add_filter("Image Files", &["png", "jpg", "jpeg", "bmp", "gif"])
        .show_open_multiple_file()
    {
        Ok(files) => files
            .into_iter()
            .map(|file| file.display().to_string())
            .collect(), // Convert paths to String
        Err(_) => vec![], // Return an empty Vec if an error occurs
    }
}