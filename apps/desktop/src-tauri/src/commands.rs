use repot_core::availability::{Availability, availability_map};
use repot_core::export::{
    ExportBundle, ExportMeta, ExportRequest, Format, build_export, write_export,
};
use repot_core::snapshot::{self, Snapshot, SnapshotError};
use repot_core::{Env, PROVIDER_TIMEOUT, Selection, Target, default_providers};
use repot_core::{os::Os, scan::ScanResult};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;

/// What the review and export screens work on: a scan of this machine or an opened
/// Repot file. Source OS, items and config contents always come from the same place.
#[derive(Clone)]
pub struct Session {
    pub source: Os,
    pub scan: ScanResult,
    pub configs: Configs,
}
#[derive(Clone)]
pub enum Configs {
    /// Read config files from this machine.
    Live,
    /// Use the contents stored in an opened Repot file.
    Snapshot(Arc<Snapshot>),
}

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
    fn preview(
        &self,
        selection: &Selection,
        target: Target,
        format: Format,
        meta: ExportMeta,
    ) -> Result<ExportBundle, String> {
        target.validate().map_err(str::to_owned)?;
        let session = self.session()?;
        let request = ExportRequest {
            items: &session.scan.items,
            selection,
            source: session.source,
            target,
            format,
            meta,
        };
        Ok(match &session.configs {
            Configs::Live => build_export(self.env.as_ref(), &request),
            Configs::Snapshot(snapshot) => build_export(snapshot.as_ref(), &request),
        })
    }
}
#[tauri::command]
pub fn detect_os(state: State<AppState>) -> Os {
    state.machine
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
    state.set_session(Session {
        source: state.machine,
        scan: result.clone(),
        configs: Configs::Live,
    })?;
    Ok(result)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedFile {
    scan: ScanResult,
    source: Os,
    created_at: String,
    repot_version: String,
    skipped_items: usize,
}

/// Error kinds the UI translates (`readFailed` covers missing or unreadable files).
fn open_error(error: SnapshotError) -> String {
    match error {
        SnapshotError::TooLarge => "tooLarge".into(),
        SnapshotError::NotRepotFile => "notRepotFile".into(),
        SnapshotError::NewerVersion { found } => format!("newerVersion:{found}"),
        SnapshotError::Invalid => "invalid".into(),
    }
}

/// Open a Repot file the user chose. On failure the current session stays as it was.
#[tauri::command]
pub fn open_snapshot(state: State<AppState>, path: String) -> Result<OpenedFile, String> {
    let path = Path::new(&path);
    // Refuse oversized files before reading them into memory.
    if std::fs::metadata(path).is_ok_and(|m| m.len() > snapshot::MAX_BYTES as u64) {
        return Err(open_error(SnapshotError::TooLarge));
    }
    let text = state
        .env
        .read_file(path)
        .map_err(|_| "readFailed".to_owned())?;
    let opened = snapshot::parse_snapshot(&text).map_err(open_error)?;
    let snapshot = Arc::new(opened.snapshot);
    let scan = ScanResult {
        items: snapshot.items.clone(),
        providers: vec![],
    };
    state.set_session(Session {
        source: snapshot.source,
        scan: scan.clone(),
        configs: Configs::Snapshot(Arc::clone(&snapshot)),
    })?;
    Ok(OpenedFile {
        scan,
        source: snapshot.source,
        created_at: snapshot.created_at.clone(),
        repot_version: snapshot.repot_version.clone(),
        skipped_items: opened.skipped_items,
    })
}

#[tauri::command]
pub fn availability_for(
    state: State<AppState>,
    target: Target,
) -> Result<HashMap<String, Availability>, String> {
    target.validate().map_err(str::to_owned)?;
    let session = state.session()?;
    Ok(availability_map(
        &session.scan.items,
        session.source,
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
