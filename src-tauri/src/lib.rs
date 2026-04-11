mod archive;
mod commands;
mod models;
mod utils;

use commands::{file_loader, image_provider, settings as settings_cmd};
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cli_args: Vec<String> = std::env::args().skip(1).collect();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            file_loader::load_source,
            image_provider::get_image,
            settings_cmd::load_settings,
            settings_cmd::save_settings,
        ])
        .setup(move |app| {
            if !cli_args.is_empty() {
                let path = cli_args[0].clone();
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let _ = handle.emit("cli-file-open", path);
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
