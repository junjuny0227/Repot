//! The Repot file (`repot.json`): written with every export, opened to review and
//! export again without scanning. See plan 11 and spec §10.

pub mod v1;

use crate::export::{ConfigSource, ConfigText};
use crate::os::Os;
use crate::providers;
use crate::scan::mark_duplicates;
use crate::{Item, ItemDetails, PackageKind, PackageManager};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};
pub use v1::StoredConfig;
use v1::{DetailsV1, ItemV1, SCHEMA_VERSION, TargetPathV1};

pub const FILE_NAME: &str = "repot.json";
pub const FORMAT: &str = "repot";
/// Opened files larger than this are rejected before parsing.
pub const MAX_BYTES: usize = 5 * 1024 * 1024;

/// An opened Repot file in model form.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub created_at: String,
    pub repot_version: String,
    pub source: Os,
    pub items: Vec<Item>,
    /// Config-file contents by item id.
    pub configs: BTreeMap<String, StoredConfig>,
    /// Ids of items the export that wrote the file could not include (plan 13).
    pub not_exported: BTreeSet<String>,
}

/// One item to write into the file.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredItem {
    pub item: Item,
    /// `Some` exactly for config-file items.
    pub config: Option<StoredConfig>,
    /// Unusable on this export's destination, kept so another destination can use it.
    pub not_exported: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenedSnapshot {
    pub snapshot: Snapshot,
    /// Items dropped because they were malformed, unknown to this version or unsafe.
    pub skipped_items: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum SnapshotError {
    #[error("file is larger than {MAX_BYTES} bytes")]
    TooLarge,
    #[error("not a Repot file")]
    NotRepotFile,
    #[error("made by a newer Repot (schema version {found})")]
    NewerVersion { found: u32 },
    #[error("invalid Repot file")]
    Invalid,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Source {
    os: Os,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileOut<'a> {
    format: &'static str,
    schema_version: u32,
    created_at: &'a str,
    repot_version: &'a str,
    source: Source,
    items: Vec<ItemV1>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Envelope {
    created_at: String,
    repot_version: String,
    source: Source,
    items: Vec<serde_json::Value>,
}

/// Render the file for an export's items and the ones its destination could not use.
pub fn render_snapshot(
    created_at: &str,
    repot_version: &str,
    source: Os,
    items: &[StoredItem],
) -> String {
    let file = FileOut {
        format: FORMAT,
        schema_version: SCHEMA_VERSION,
        created_at,
        repot_version,
        source: Source { os: source },
        items: items
            .iter()
            .map(|s| ItemV1::from_item(&s.item, s.config.as_ref(), s.not_exported))
            .collect(),
    };
    let mut text = serde_json::to_string_pretty(&file).expect("Repot file serializes");
    text.push('\n');
    text
}

/// Parse an untrusted Repot file. Unsafe or unknown items are skipped and counted.
pub fn parse_snapshot(text: &str) -> Result<OpenedSnapshot, SnapshotError> {
    if text.len() > MAX_BYTES {
        return Err(SnapshotError::TooLarge);
    }
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|_| SnapshotError::NotRepotFile)?;
    if value.get("format").and_then(|v| v.as_str()) != Some(FORMAT) {
        return Err(SnapshotError::NotRepotFile);
    }
    let version = value
        .get("schemaVersion")
        .and_then(|v| v.as_u64())
        .ok_or(SnapshotError::Invalid)?;
    if version > u64::from(SCHEMA_VERSION) {
        return Err(SnapshotError::NewerVersion {
            found: u32::try_from(version).unwrap_or(u32::MAX),
        });
    }
    if version != u64::from(SCHEMA_VERSION) {
        return Err(SnapshotError::Invalid);
    }
    let envelope: Envelope = serde_json::from_value(value).map_err(|_| SnapshotError::Invalid)?;
    let mut ids = HashSet::new();
    let mut items = vec![];
    let mut configs = BTreeMap::new();
    let mut not_exported = BTreeSet::new();
    let mut skipped_items = 0;
    for raw in envelope.items {
        let Ok(item) = serde_json::from_value::<ItemV1>(raw) else {
            skipped_items += 1;
            continue;
        };
        if !is_safe_item(&item) || !ids.insert(item.id.clone()) {
            skipped_items += 1;
            continue;
        }
        let not_exported_item = item.not_exported;
        let (item, config) = item.into_item();
        if not_exported_item {
            not_exported.insert(item.id.clone());
        }
        if let Some(config) = config {
            configs.insert(item.id.clone(), config);
        }
        items.push(item);
    }
    mark_duplicates(&mut items);
    Ok(OpenedSnapshot {
        snapshot: Snapshot {
            created_at: envelope.created_at,
            repot_version: envelope.repot_version,
            source: envelope.source.os,
            items,
            configs,
            not_exported,
        },
        skipped_items,
    })
}

/// Paths must stay inside the destination folder and, for config files and plugins, at a
/// place the item's provider scans; plugin URLs must be https.
/// Package names and versions are checked again by the export like scanned values.
fn is_safe_item(item: &ItemV1) -> bool {
    if item.id.is_empty() || item.name.is_empty() {
        return false;
    }
    match &item.details {
        // The URL is passed to `scoop bucket add`; only custom Scoop buckets carry one.
        DetailsV1::Package {
            manager,
            kind,
            url: Some(url),
        } => {
            *manager == PackageManager::Scoop
                && *kind == PackageKind::Tap
                && crate::export::step::is_safe_bucket_url(url)
        }
        DetailsV1::ConfigFile { target, .. } => {
            is_safe_target(target) && is_scanned_config_target(&item.provider, target)
        }
        // The folder name is joined to the plugins folder by the restore script.
        DetailsV1::IdePlugin { folder, .. } => {
            crate::export::step::is_safe_value(folder)
                && !folder.contains('/')
                && !folder.contains(':')
                && folder != "."
                && folder != ".."
        }
        // Passed to `rustup component add` / `rustup target add` by the restore script.
        DetailsV1::Runtime {
            components,
            targets,
            ..
        } => components
            .iter()
            .chain(targets)
            .all(|name| crate::providers::rustup::is_rustup_name(name)),
        DetailsV1::ShellPlugin { url, target } => {
            url.starts_with("https://")
                && crate::export::step::is_safe_value(url)
                && is_safe_target(target)
                && item.provider == "shell"
                && matches!(target, TargetPathV1::Home(rel) if providers::shell::is_plugin_target(&rel.replace('\\', "/")))
        }
        _ => true,
    }
}

fn is_safe_target(target: &TargetPathV1) -> bool {
    if let TargetPathV1::JetBrains { version, .. } = target
        && !crate::jetbrains::is_version(version)
    {
        return false;
    }
    let Some(rel) = target.relative() else {
        return true;
    };
    let rel = rel.replace('\\', "/");
    !rel.is_empty()
        && !rel.starts_with('/')
        && !rel.starts_with('~')
        && !rel.contains(':')
        && rel.split('/').all(|part| !part.is_empty() && part != "..")
}

/// A config file may only go where the provider named in the item writes its files on a
/// scan. Being inside home is not enough: `.ssh/authorized_keys`, `Library/LaunchAgents/`
/// or `.config/autostart/` are inside home too, and Repot never writes there. Only `ssh`
/// accepts `.ssh/`, and only its config files (plan 16).
fn is_scanned_config_target(provider: &str, target: &TargetPathV1) -> bool {
    let rel = target.relative().map(|rel| rel.replace('\\', "/"));
    let rel = rel.as_deref().unwrap_or_default();
    match (provider, target) {
        ("ssh", TargetPathV1::Home(_)) => providers::ssh::is_config_target(rel),
        ("git", TargetPathV1::Home(_)) => providers::git::is_config_target(rel),
        ("shell", TargetPathV1::Home(_)) => providers::shell::is_config_target(rel),
        ("shell", TargetPathV1::PowerShellProfile) => true,
        ("vscode", TargetPathV1::EditorUser { .. }) => providers::vscode::is_config_target(rel),
        ("jetbrains", TargetPathV1::JetBrains { .. }) => {
            providers::jetbrains::is_config_target(rel)
        }
        _ => false,
    }
}

impl ConfigSource for Snapshot {
    fn read_config(&self, item: &Item) -> Option<ConfigText> {
        if !matches!(item.details, ItemDetails::ConfigFile { .. }) {
            return None;
        }
        let stored = self.configs.get(&item.id)?;
        Some(ConfigText {
            text: stored.content.clone(),
            masked: stored.masked.clone(),
            revealed: Some(stored.revealed.clone()),
        })
    }
}
