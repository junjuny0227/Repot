pub mod manifest;
pub mod meta;
pub use meta::ExportMeta;

use crate::{
    Env, Item, ItemDetails, Selection,
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
    pub target: Os,
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
            let file = payload::prepare_config(item, &content, &reveal, req.source, req.target);
            payloads.push(OutputFile {
                path: format!("files/{}", file.path),
                contents: file.contents,
                executable: false,
            });
            masked.extend(file.masked);
            notes.extend(file.notes);
        } else {
            let extra = match &item.details {
                ItemDetails::ShellPlugin { url, .. } => Some(url.as_str()),
                ItemDetails::Runtime { runtime, .. } => Some(runtime.as_str()),
                _ => None,
            };
            if !step::is_safe_value(&item.name)
                || item
                    .version
                    .as_deref()
                    .is_some_and(|v| !step::is_safe_value(v))
                || extra.is_some_and(|v| !step::is_safe_value(v))
            {
                warnings.push(ExportWarning::InvalidValue { item_id });
                continue;
            }
        }
        steps.extend(step::steps_for(item, req.target));
    }
    step::sort_steps(&mut steps);
    let header = ScriptHeader {
        repot_version: req.meta.repot_version.clone(),
        source: req.source,
        target: req.target,
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
            let windows = req.target == Os::Windows;
            vec![OutputFile {
                path: if windows { "restore.ps1" } else { "restore.sh" }.into(),
                contents: if windows {
                    script::powershell::render_powershell(&plan)
                } else {
                    script::bash::render_bash(&plan)
                },
                executable: !windows,
            }]
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
