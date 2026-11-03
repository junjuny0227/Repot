use repot_core::availability::{Availability, availability_map};
use repot_core::export::{
    ExportBundle, ExportMeta, ExportRequest, Format, build_export, write_export,
};
use repot_core::{Env, PROVIDER_TIMEOUT, Selection, Target, default_providers};
use repot_core::{os::Os, scan::ScanResult};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;

pub struct AppState {
    pub env: Arc<dyn Env>,
    pub source: Os,
    pub scan: Mutex<Option<ScanResult>>,
    pub last_export: Mutex<Option<PathBuf>>,
}
impl AppState {
    fn scan_result(&self) -> Result<ScanResult, String> {
        self.scan
            .lock()
            .map_err(|e| e.to_string())?
            .clone()
            .ok_or_else(|| "no scan result".into())
    }
    fn preview(
        &self,
        selection: &Selection,
        target: Target,
        format: Format,
        meta: ExportMeta,
    ) -> Result<ExportBundle, String> {
        target.validate().map_err(str::to_owned)?;
        let scan = self.scan_result()?;
        Ok(build_export(
            self.env.as_ref(),
            &ExportRequest {
                items: &scan.items,
                selection,
                source: self.source,
                target,
                format,
                meta,
            },
        ))
    }
}
#[tauri::command]
pub fn detect_os(state: State<AppState>) -> Os {
    state.source
}

#[tauri::command]
pub async fn scan(app: AppHandle, state: State<'_, AppState>) -> Result<ScanResult, String> {
    let env = Arc::clone(&state.env);
    let result = tauri::async_runtime::spawn_blocking(move || {
        repot_core::scan(env, default_providers(), PROVIDER_TIMEOUT, &|report| {
            let _ = app.emit("scan://progress", report);
        })
    })
    .await
    .map_err(|e| e.to_string())?;
    *state.scan.lock().map_err(|e| e.to_string())? = Some(result.clone());
    Ok(result)
}
#[tauri::command]
pub fn availability_for(
    state: State<AppState>,
    target: Target,
) -> Result<HashMap<String, Availability>, String> {
    target.validate().map_err(str::to_owned)?;
    Ok(availability_map(
        &state.scan_result()?.items,
        state.source,
        target,
    ))
}
#[tauri::command]
pub fn preview_export(
    state: State<AppState>,
    selection: Selection,
    target: Target,
    format: Format,
) -> Result<ExportBundle, String> {
    state.preview(&selection, target, format, ExportMeta::now())
}
#[tauri::command]
pub fn export(
    state: State<AppState>,
    selection: Selection,
    target: Target,
    format: Format,
    parent: String,
) -> Result<String, String> {
    let meta = ExportMeta::now();
    let bundle = state.preview(&selection, target, format, meta.clone())?;
    let path = write_export(&bundle, Path::new(&parent), &meta.date).map_err(|e| e.to_string())?;
    *state.last_export.lock().map_err(|e| e.to_string())? = Some(path.clone());
    Ok(path.to_string_lossy().into_owned())
}
#[tauri::command]
pub fn open_export_folder(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let path = state
        .last_export
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("no export folder")?;
    app.opener()
        .open_path(path.to_string_lossy().into_owned(), None::<&str>)
        .map_err(|e| e.to_string())
}
