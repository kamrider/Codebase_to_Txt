mod application;
mod commands;
mod domain;
mod infrastructure;
mod models;

use commands::{evaluate_selection, preview_export, run_export, scan_children, scan_tree};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_tree,
            scan_children,
            evaluate_selection,
            preview_export,
            run_export
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
