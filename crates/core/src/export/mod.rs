pub mod manifest;
pub mod meta;
pub use meta::ExportMeta;

use crate::{
    Env, Item, ItemDetails, Selection, Target,
    availability::{Availability, UnavailableReason, availability},
    os::Os,
};
use script::{ScriptHeader, ScriptPlan};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Format {
    Script,
    Manifest,
}

pub struct ExportRequest<'a> {
    pub items: &'a [Item],
    pub selection: &'a Selection,
    pub source: Os,
    pub target: Target,
    pub format: Format,
    pub meta: ExportMeta,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ExportWarning {
    Unavailable {
        item_id: String,
        reason: UnavailableReason,
    },
    InvalidValue {
        item_id: String,
    },
    DuplicateTarget {
        item_id: String,
    },
    ReadFailed {
        item_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportBundle {
    pub files: Vec<OutputFile>,
    pub warnings: Vec<ExportWarning>,
}

/// Assemble an in-memory preview; item failures become warnings, never a failed export.
pub fn build_export(env: &dyn Env, req: &ExportRequest<'_>) -> ExportBundle {
    let selected: HashSet<_> = req.selection.items.iter().collect();
    let mut targets = HashSet::new();
    let mut package_targets = HashSet::new();
    let mut taps = HashSet::new();
    let mut warnings = vec![];
    let mut payloads = vec![];
    let mut steps = vec![];
    let mut masked = vec![];
    let mut notes = vec![];
    for item in req.items.iter().filter(|item| selected.contains(&item.id)) {
        let item_id = item.id.clone();
        if let Availability::Unavailable(reason) = availability(item, req.source, req.target) {
            warnings.push(ExportWarning::Unavailable { item_id, reason });
            continue;
        }
        if let ItemDetails::ConfigFile { source, target } = &item.details {
            if !targets.insert(paths::payload_path(target)) {
                warnings.push(ExportWarning::DuplicateTarget { item_id });
                continue;
            }
            let Ok(content) = env.read_file(source) else {
                warnings.push(ExportWarning::ReadFailed { item_id });
                continue;
            };
            let reveal: Vec<_> = req
                .selection
                .reveal_secret_lines
                .iter()
                .filter(|(id, _)| id == &item.id)
                .map(|(_, line)| *line)
                .collect();
            let file = payload::prepare_config(item, &content, &reveal, req.source, req.target.os);
            payloads.push(OutputFile {
                path: format!("files/{}", file.path),
                contents: file.contents,
                executable: false,
            });
            masked.extend(file.masked);
            notes.extend(file.notes);
        } else {
            // Only values that end up in the output are validated. Packages, globals and
            // extensions install the latest version, so their scanned version (e.g. a brew
            // cask's "17.0.21,17.70") is never written and must not block the item.
            // Runtime names are display labels ("Rust toolchain"); steps use tool, runtime and
            // version, so the name is not validated either.
            let (name, extra, version) = match &item.details {
                ItemDetails::ShellPlugin { url, .. } => {
                    (Some(item.name.as_str()), Some(url.as_str()), None)
                }
                ItemDetails::Runtime { runtime, .. } => {
                    (None, Some(runtime.as_str()), item.version.as_deref())
                }
                _ => (Some(item.name.as_str()), None, None),
            };
            if name.is_some_and(|v| !step::is_safe_value(v))
                || version.is_some_and(|v| !step::is_safe_value(v))
                || extra.is_some_and(|v| !step::is_safe_value(v))
            {
                warnings.push(ExportWarning::InvalidValue { item_id });
                continue;
            }
        }
        let item_steps = step::steps_for_target(item, req.source, req.target);
        if let Some(step::Step::InstallPackage {
            manager,
            kind,
            name,
            mapped_from: Some(_),
            ..
        }) = item_steps
            .iter()
            .find(|s| matches!(s, step::Step::InstallPackage { .. }))
        {
            let normalized = if *manager == crate::PackageManager::Scoop {
                name.strip_prefix("main/").unwrap_or(name)
            } else {
                name
            };
            if !package_targets.insert((*manager, *kind, normalized.to_ascii_lowercase())) {
                warnings.push(ExportWarning::DuplicateTarget { item_id });
                continue;
            }
        }
        // Validate mapped names as well as source values before rendering.
        if item_steps.iter().any(|s| matches!(s, step::Step::InstallPackage { name, .. } | step::Step::AddTap { name, .. } if !step::is_safe_value(name))) {
            warnings.push(ExportWarning::InvalidValue { item_id });
            continue;
        }
        steps.extend(item_steps.into_iter().filter(|s| match s {
            step::Step::AddTap { manager, name } if req.source != req.target.os => {
                taps.insert((*manager, name.to_ascii_lowercase()))
            }
            _ => true,
        }));
    }
    step::sort_steps(&mut steps);
    let header = ScriptHeader {
        repot_version: req.meta.repot_version.clone(),
        source: req.source,
        target: req.target.os,
        generated_at: req.meta.generated_at.clone(),
    };
    let mut files = match req.format {
        Format::Manifest => manifest::render_manifests(&header, &steps, &masked, &notes),
        Format::Script => {
            let plan = ScriptPlan {
                header,
                steps,
                masked,
                notes,
            };
            let windows = req.target.os == Os::Windows;
            let mappings = manifest::mapped_packages(&plan.steps);
            let mut files = vec![OutputFile {
                path: if windows { "restore.ps1" } else { "restore.sh" }.into(),
                contents: if windows {
                    script::powershell::render_powershell(&plan)
                } else {
                    script::bash::render_bash(&plan)
                },
                executable: !windows,
            }];
            if !mappings.is_empty() {
                files.push(OutputFile { path: "README.md".into(), contents: format!("# Repot export\n\nReview the restore script before running it. Install the required package managers first.\n{mappings}"), executable: false });
            }
            files
        }
    };
    payloads.sort_by(|a, b| a.path.cmp(&b.path));
    files.extend(payloads);
    ExportBundle { files, warnings }
}
pub mod paths;
pub mod payload;
pub mod script;
pub mod step;
pub mod write;
pub use write::{ExportError, write_export};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputFile {
    pub path: String,
    pub contents: String,
    pub executable: bool,
}
