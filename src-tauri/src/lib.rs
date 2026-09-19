pub mod commands;
pub mod compare;
pub mod error;
pub mod excel;
pub mod models;
pub mod parser;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct AppState {
    pub cancel: Arc<AtomicBool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            cancel: Arc::new(AtomicBool::new(false)),
        })
        .invoke_handler(tauri::generate_handler![
            commands::inspect_file::inspect_file,
            commands::compare_files::compare_files,
            commands::export_excel::export_excel,
            commands::cancel_compare::cancel_compare,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
