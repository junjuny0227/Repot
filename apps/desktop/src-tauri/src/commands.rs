use repot_core::availability::Availability;
use repot_core::compare::Presence;
use repot_core::export::{ExportBundle, ExportMeta, Format, write_export};
use repot_core::session::{OpenError, Session};
use repot_core::snapshot::SnapshotError;
use repot_core::{Env, PROVIDER_TIMEOUT, Selection, Target, default_providers};
use repot_core::{os::Os, scan::ScanResult};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;

/// The review session itself lives in core (shared with the CLI, plan 15); this layer
/// keeps the current one and turns its results into command responses.
pub struct AppState {
    pub env: Arc<dyn Env>,
    /// The OS this app runs on; the source of a new scan.
    pub machine: Os,
    pub session: Mutex<Option<Session>>,
    pub last_export: Mutex<Option<PathBuf>>,
}
impl AppState {
    fn session(&self) -> Result<Session, String> {
        self.session
            .lock()
            .map_err(|e| e.to_string())?
            .clone()
            .ok_or_else(|| "no scan result".into())
    }
    fn set_session(&self, session: Session) -> Result<(), String> {
        *self.session.lock().map_err(|e| e.to_string())? = Some(session);
        Ok(())
    }
    fn update_session(&self, change: impl FnOnce(&mut Session)) -> Result<(), String> {
        let mut session = self.session.lock().map_err(|e| e.to_string())?;
        change(session.as_mut().ok_or("no scan result")?);
        Ok(())
    }
    fn preview(
        &self,
        selection: &Selection,
        target: Target,
        format: Format,
        meta: ExportMeta,
    ) -> Result<ExportBundle, String> {
        self.session()?
            .preview(self.env.as_ref(), selection, target, format, meta)
            .map_err(str::to_owned)
    }
}

/// Scan this machine off the main thread, reporting progress as `scan://progress`.
async fn scan_this_machine(app: AppHandle, env: Arc<dyn Env>) -> Result<ScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        repot_core::scan(env, default_providers(), PROVIDER_TIMEOUT, &|report| {
            let _ = app.emit("scan://progress", report);
        })
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn detect_os(state: State<AppState>) -> Os {
    state.machine
}

#[tauri::command]
pub async fn scan(app: AppHandle, state: State<'_, AppState>) -> Result<ScanResult, String> {
    let result = scan_this_machine(app, Arc::clone(&state.env)).await?;
    state.set_session(Session::from_scan(state.machine, result.clone()))?;
    Ok(result)
}

/// Scan this machine to compare the current session (an opened Repot file) with it.
/// The session's own items stay as they are.
#[tauri::command]
pub async fn scan_machine(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.session()?;
    let result = scan_this_machine(app, Arc::clone(&state.env)).await?;
    state.update_session(|session| session.machine = Some(result))
}

/// Presence of each item on this machine for `target`. Empty unless this machine was
/// scanned for comparison and `target` is this machine's OS.
#[tauri::command]
pub fn presence_for(
    state: State<AppState>,
    target: Target,
) -> Result<HashMap<String, Presence>, String> {
    target.validate().map_err(str::to_owned)?;
    Ok(state
        .session()?
        .presence(state.env.as_ref(), state.machine, target))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedFile {
    scan: ScanResult,
    source: Os,
    created_at: String,
    repot_version: String,
    skipped_items: usize,
    /// Items the export that wrote the file could not include (plan 13).
    not_exported: Vec<String>,
}

/// Error kinds the UI translates (`readFailed` covers missing or unreadable files).
fn open_error(error: OpenError) -> String {
    match error {
        OpenError::ReadFailed => "readFailed".into(),
        OpenError::Snapshot(SnapshotError::TooLarge) => "tooLarge".into(),
        OpenError::Snapshot(SnapshotError::NotRepotFile) => "notRepotFile".into(),
        OpenError::Snapshot(SnapshotError::NewerVersion { found }) => {
            format!("newerVersion:{found}")
        }
        OpenError::Snapshot(SnapshotError::Invalid) => "invalid".into(),
    }
}

/// Open a Repot file the user chose. On failure the current session stays as it was.
#[tauri::command]
pub fn open_snapshot(state: State<AppState>, path: String) -> Result<OpenedFile, String> {
    let (session, info) =
        Session::open(state.env.as_ref(), Path::new(&path)).map_err(open_error)?;
    let opened = OpenedFile {
        scan: session.scan.clone(),
        source: session.source,
        created_at: info.created_at,
        repot_version: info.repot_version,
        skipped_items: info.skipped_items,
        not_exported: session.not_exported().iter().cloned().collect(),
    };
    state.set_session(session)?;
    Ok(opened)
}

#[tauri::command]
pub fn availability_for(
    state: State<AppState>,
    target: Target,
) -> Result<HashMap<String, Availability>, String> {
    target.validate().map_err(str::to_owned)?;
    Ok(state.session()?.availability(target))
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
