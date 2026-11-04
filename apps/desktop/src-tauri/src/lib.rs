mod commands;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePlatform {
    platform: &'static str,
    app_image: bool,
}

#[tauri::command]
fn update_platform(_app: tauri::AppHandle) -> UpdatePlatform {
    #[cfg(target_os = "linux")]
    {
        use tauri::Manager;
        UpdatePlatform {
            platform: "linux",
            app_image: _app.env().appimage.is_some(),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        UpdatePlatform {
            platform: std::env::consts::OS,
            app_image: false,
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .manage(commands::AppState {
            env: std::sync::Arc::new(repot_core::RealEnv::new()),
            machine: repot_core::os::current_os(),
            session: std::sync::Mutex::new(None),
            last_export: std::sync::Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            update_platform,
            commands::detect_os,
            commands::scan,
            commands::open_snapshot,
            commands::scan_machine,
            commands::presence_for,
            commands::availability_for,
            commands::preview_export,
            commands::export,
            commands::open_export_folder
        ])
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running Repot");
}
