mod commands;

pub fn run() {
    tauri::Builder::default()
        .manage(commands::AppState {
            env: std::sync::Arc::new(repot_core::RealEnv::new()),
            source: repot_core::os::current_os(),
            scan: std::sync::Mutex::new(None),
            last_export: std::sync::Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_os,
            commands::scan,
            commands::availability_for,
            commands::preview_export,
            commands::export,
            commands::open_export_folder
        ])
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running Repot");
}
