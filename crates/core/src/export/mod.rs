pub mod manifest;
pub mod meta;
pub use meta::ExportMeta;

use crate::{
    Env, Item, ItemDetails, Selection, Target,
    availability::{Availability, UnavailableReason, availability},
    os::Os,
    snapshot,
};
use script::{ScriptHeader, ScriptPlan};
use std::collections::{HashMap, HashSet};

/// Where an export reads config-file contents: the scanned machine (`Env`) or an
/// opened Repot file.
pub struct ConfigText {
    pub text: String,
    /// Lines already masked in `text` (an opened file), with the secret's name if known.
    pub masked: Vec<(u32, Option<String>)>,
    /// Lines to keep as written. `None` means use the request's selection.
    pub revealed: Option<Vec<u32>>,
}
pub trait ConfigSource {
    /// `None` when the item has no readable config contents.
    fn read_config(&self, item: &Item) -> Option<ConfigText>;
}
impl<E: Env + ?Sized> ConfigSource for E {
    fn read_config(&self, item: &Item) -> Option<ConfigText> {
        let ItemDetails::ConfigFile { source, .. } = &item.details else {
            return None;
        };
        Some(ConfigText {
            text: self.read_file(source).ok()?,
            masked: vec![],
            revealed: None,
        })
    }
}

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
/// The bundle always includes `repot.json` with exactly the items it exports.
pub fn build_export(
    configs: &(impl ConfigSource + ?Sized),
    req: &ExportRequest<'_>,
) -> ExportBundle {
    let selected: HashSet<_> = req.selection.items.iter().collect();
    let mut targets = HashSet::new();
    let mut package_targets = HashSet::new();
    let mut taps = HashSet::new();
    let mut warnings = vec![];
    let mut payloads = vec![];
    let mut steps = vec![];
    let mut masked = vec![];
    let mut notes = vec![];
    let mut snapshot_items = vec![];
    let mut unusable_items = vec![];
    // Items this destination cannot use go into the Repot file only (plan 13), so the file
    // can still be exported for another destination. Unchecked usable items stay out.
    for item in req.items {
        if let Availability::Unavailable(reason) = availability(item, req.source, req.target)
            && reason != UnavailableReason::Unreadable
            && let Some(stored) = not_exported_item(configs, req, item)
        {
            unusable_items.push(stored);
        }
    }
    for item in req.items.iter().filter(|item| selected.contains(&item.id)) {
        let item_id = item.id.clone();
        let mut pending_config = None;
        if let Availability::Unavailable(reason) = availability(item, req.source, req.target) {
            warnings.push(ExportWarning::Unavailable { item_id, reason });
            continue;
        }
        if let ItemDetails::ConfigFile { target, .. } = &item.details {
            if !targets.insert(paths::payload_path(target)) {
                warnings.push(ExportWarning::DuplicateTarget { item_id });
                continue;
            }
            let Some(config) = configs.read_config(item) else {
                warnings.push(ExportWarning::ReadFailed { item_id });
                continue;
            };
            let reveal = reveal_lines(req, item, &config);
            let mut file =
                payload::prepare_config(item, &config.text, &reveal, req.source, req.target.os);
            merge_masked(&mut file.masked, &file.path, &config.masked);
            pending_config = Some(stored_config(item, &config, reveal, req.source));
            payloads.push(OutputFile {
                path: format!("files/{}", file.path),
                contents: file.contents,
                executable: false,
            });
            masked.extend(file.masked);
            notes.extend(file.notes);
        } else {
            if !values_are_safe(item) {
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
        snapshot_items.push(snapshot::StoredItem {
            item: item.clone(),
            config: pending_config,
            not_exported: false,
        });
        steps.extend(item_steps.into_iter().filter(|s| match s {
            step::Step::AddTap { manager, name, .. } if req.source != req.target.os => {
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
    // Keep the scan's order in the file whether or not an item was exported.
    snapshot_items.extend(unusable_items);
    let order: HashMap<&str, usize> = req
        .items
        .iter()
        .enumerate()
        .rev()
        .map(|(n, item)| (item.id.as_str(), n))
        .collect();
    snapshot_items.sort_by_key(|stored| order[stored.item.id.as_str()]);
    files.push(OutputFile {
        path: snapshot::FILE_NAME.into(),
        contents: snapshot::render_snapshot(
            &req.meta.created_at(),
            &req.meta.repot_version,
            req.source,
            &snapshot_items,
        ),
        executable: false,
    });
    payloads.sort_by(|a, b| a.path.cmp(&b.path));
    files.extend(payloads);
    ExportBundle { files, warnings }
}
/// Only values that end up in the output are validated. Packages, globals and extensions
/// install the latest version, so their scanned version (e.g. a brew cask's "17.0.21,17.70")
/// is never written and must not block the item. Runtime names are display labels ("Rust
/// toolchain"); steps use tool, runtime and version, so the name is not validated either.
fn values_are_safe(item: &Item) -> bool {
    let (name, extra, version) = match &item.details {
        ItemDetails::ConfigFile { .. } => return true,
        ItemDetails::Package { url: Some(url), .. } => {
            if !step::is_safe_bucket_url(url) {
                return false;
            }
            (Some(item.name.as_str()), None, None)
        }
        ItemDetails::ShellPlugin { url, .. } => {
            (Some(item.name.as_str()), Some(url.as_str()), None)
        }
        ItemDetails::Runtime {
            runtime,
            components,
            targets,
            ..
        } => {
            if !components
                .iter()
                .chain(targets)
                .all(|name| crate::providers::rustup::is_rustup_name(name))
            {
                return false;
            }
            (None, Some(runtime.as_str()), item.version.as_deref())
        }
        // The folder name ends up in the "already installed" check.
        ItemDetails::IdePlugin { folder, .. } => {
            (Some(item.name.as_str()), Some(folder.as_str()), None)
        }
        _ => (Some(item.name.as_str()), None, None),
    };
    [name, extra, version]
        .into_iter()
        .flatten()
        .all(step::is_safe_value)
}

/// A config file in the source machine's form, secrets masked except revealed lines, so
/// the Repot file can target any OS later.
fn stored_config(
    item: &Item,
    config: &ConfigText,
    reveal: Vec<u32>,
    source: Os,
) -> snapshot::StoredConfig {
    let mut stored = payload::prepare_config(item, &config.text, &reveal, source, source);
    merge_masked(&mut stored.masked, &stored.path, &config.masked);
    snapshot::StoredConfig {
        content: stored.contents,
        masked: stored
            .masked
            .into_iter()
            .map(|m| (m.line, m.name))
            .collect(),
        revealed: reveal,
    }
}

/// An item this destination cannot use, stored with the same rules as exported items.
/// `None` when it cannot be stored safely (unreadable config, unsafe value).
fn not_exported_item(
    configs: &(impl ConfigSource + ?Sized),
    req: &ExportRequest<'_>,
    item: &Item,
) -> Option<snapshot::StoredItem> {
    if !values_are_safe(item) {
        return None;
    }
    let config = match &item.details {
        ItemDetails::ConfigFile { .. } => {
            let config = configs.read_config(item)?;
            let reveal = reveal_lines(req, item, &config);
            Some(stored_config(item, &config, reveal, req.source))
        }
        _ => None,
    };
    Some(snapshot::StoredItem {
        item: item.clone(),
        config,
        not_exported: true,
    })
}

/// Lines shown as written: an opened file's record, else the user's choice in this session.
fn reveal_lines(req: &ExportRequest<'_>, item: &Item, config: &ConfigText) -> Vec<u32> {
    config.revealed.clone().unwrap_or_else(|| {
        req.selection
            .reveal_secret_lines
            .iter()
            .filter(|(id, _)| id == &item.id)
            .map(|(_, line)| *line)
            .collect()
    })
}
/// Add masked lines an opened file already recorded; detection cannot see a bare `<REDACTED>`.
fn merge_masked(
    masked: &mut Vec<payload::MaskedLine>,
    file: &str,
    known: &[(u32, Option<String>)],
) {
    for (line, name) in known {
        match masked.iter_mut().find(|m| m.line == *line) {
            Some(existing) => {
                if existing.name.is_none() {
                    existing.name = name.clone();
                }
            }
            None => masked.push(payload::MaskedLine {
                file: file.into(),
                line: *line,
                name: name.clone(),
            }),
        }
    }
    masked.sort_by_key(|m| m.line);
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
