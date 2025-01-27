pub mod command_handler;
pub mod cpp;
pub mod rs;

use command_handler::GenerateInvokeHandler;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .generate_invoke_handler()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
