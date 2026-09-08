#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod pipeline;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(pipeline::AppState::new())
        .invoke_handler(tauri::generate_handler![
            pipeline::list_specs,
            pipeline::process_photo,
            pipeline::adjust_crop,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenPhotoId");
}
