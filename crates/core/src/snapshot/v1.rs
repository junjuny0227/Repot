//! Version 1 of the Repot file (`repot.json`).
//!
//! These types are the file format, not the app's model. Only `from_item` and
//! `into_item` know `Item`, so the model can change without breaking files people
//! already have. Leaf enums (`Category`, `PackageManager`, `Portability`, ...) are
//! reused from the model; the committed fixtures in `tests/fixtures/snapshot/` pin
//! their spelling. If one of them has to change, copy its v1 spelling here first.
//!
//! Rules: within schema version 1, only add optional fields or enum values. Removing
//! or renaming anything means version 2 plus a v1 → v2 conversion; never drop this reader.

use crate::model::{
    Category, Editor, Finding, GlobalManager, Item, ItemDetails, JetBrainsIde, Origin, PackageKind,
    PackageManager, Portability, RuntimeTool, TargetPath,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemV1 {
    pub id: String,
    pub provider: String,
    pub category: Category,
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub catalog_id: Option<String>,
    pub origin: Origin,
    pub portability: Portability,
    pub details: DetailsV1,
    #[serde(default)]
    pub findings: Vec<Finding>,
    /// The export that wrote this file could not include the item on its destination
    /// (plan 13). Written only when true.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub not_exported: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum DetailsV1 {
    Package {
        manager: PackageManager,
        kind: PackageKind,
    },
    Runtime {
        tool: RuntimeTool,
        runtime: String,
        is_default: bool,
    },
    GlobalPackage {
        manager: GlobalManager,
    },
    Extension {
        editor: Editor,
    },
    /// Added in 0.7 (plan 14).
    IdePlugin {
        ide: JetBrainsIde,
        folder: String,
    },
    ConfigFile {
        source: String,
        target: TargetPathV1,
        /// Source-machine form; secrets replaced with `<REDACTED>` except revealed lines.
        content: String,
        #[serde(default)]
        masked_lines: Vec<MaskedLineV1>,
        #[serde(default)]
        revealed_lines: Vec<u32>,
    },
    ShellPlugin {
        url: String,
        target: TargetPathV1,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum TargetPathV1 {
    Home(String),
    EditorUser {
        editor: Editor,
        rel: String,
    },
    PowerShellProfile,
    /// Added in 0.7 (plan 14).
    JetBrains {
        ide: JetBrainsIde,
        version: String,
        rel: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MaskedLineV1 {
    pub line: u32,
    #[serde(default)]
    pub name: Option<String>,
}

/// A config file's stored text and secret bookkeeping.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StoredConfig {
    pub content: String,
    pub masked: Vec<(u32, Option<String>)>,
    pub revealed: Vec<u32>,
}

fn path_string(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

impl TargetPathV1 {
    fn from_model(target: &TargetPath) -> Self {
        match target {
            TargetPath::Home(rel) => Self::Home(path_string(rel)),
            TargetPath::EditorUser { editor, rel } => Self::EditorUser {
                editor: *editor,
                rel: path_string(rel),
            },
            TargetPath::PowerShellProfile => Self::PowerShellProfile,
            TargetPath::JetBrains { ide, version, rel } => Self::JetBrains {
                ide: *ide,
                version: version.clone(),
                rel: path_string(rel),
            },
        }
    }
    fn into_model(self) -> TargetPath {
        match self {
            Self::Home(rel) => TargetPath::Home(PathBuf::from(rel)),
            Self::EditorUser { editor, rel } => TargetPath::EditorUser {
                editor,
                rel: PathBuf::from(rel),
            },
            Self::PowerShellProfile => TargetPath::PowerShellProfile,
            Self::JetBrains { ide, version, rel } => TargetPath::JetBrains {
                ide,
                version,
                rel: PathBuf::from(rel),
            },
        }
    }
    /// Relative path inside home or the editor's user folder, if any.
    pub fn relative(&self) -> Option<&str> {
        match self {
            Self::Home(rel) | Self::EditorUser { rel, .. } | Self::JetBrains { rel, .. } => {
                Some(rel)
            }
            Self::PowerShellProfile => None,
        }
    }
}

impl ItemV1 {
    /// `config` must be `Some` exactly for config-file items.
    pub fn from_item(item: &Item, config: Option<&StoredConfig>, not_exported: bool) -> Self {
        let details = match (&item.details, config) {
            (ItemDetails::Package { manager, kind }, _) => DetailsV1::Package {
                manager: *manager,
                kind: *kind,
            },
            (
                ItemDetails::Runtime {
                    tool,
                    runtime,
                    is_default,
                },
                _,
            ) => DetailsV1::Runtime {
                tool: *tool,
                runtime: runtime.clone(),
                is_default: *is_default,
            },
            (ItemDetails::GlobalPackage { manager }, _) => {
                DetailsV1::GlobalPackage { manager: *manager }
            }
            (ItemDetails::Extension { editor }, _) => DetailsV1::Extension { editor: *editor },
            (ItemDetails::IdePlugin { ide, folder }, _) => DetailsV1::IdePlugin {
                ide: *ide,
                folder: folder.clone(),
            },
            (ItemDetails::ConfigFile { source, target }, config) => {
                let config = config.cloned().unwrap_or_default();
                DetailsV1::ConfigFile {
                    source: path_string(source),
                    target: TargetPathV1::from_model(target),
                    content: config.content,
                    masked_lines: config
                        .masked
                        .into_iter()
                        .map(|(line, name)| MaskedLineV1 { line, name })
                        .collect(),
                    revealed_lines: config.revealed,
                }
            }
            (ItemDetails::ShellPlugin { url, target }, _) => DetailsV1::ShellPlugin {
                url: url.clone(),
                target: TargetPathV1::from_model(target),
            },
        };
        Self {
            id: item.id.clone(),
            provider: item.provider.clone(),
            category: item.category,
            name: item.name.clone(),
            version: item.version.clone(),
            description: item.description.clone(),
            catalog_id: item.catalog_id.clone(),
            origin: item.origin.clone(),
            portability: item.portability.clone(),
            details,
            // Duplicates depend on the other items on the machine; recomputed on open.
            // Unreadable items are never exported.
            findings: item
                .findings
                .iter()
                .filter(|f| {
                    !matches!(
                        f.kind,
                        crate::model::FindingKind::Duplicate { .. }
                            | crate::model::FindingKind::Unreadable
                    )
                })
                .cloned()
                .collect(),
            not_exported,
        }
    }

    pub fn into_item(self) -> (Item, Option<StoredConfig>) {
        let (details, config) = match self.details {
            DetailsV1::Package { manager, kind } => (ItemDetails::Package { manager, kind }, None),
            DetailsV1::Runtime {
                tool,
                runtime,
                is_default,
            } => (
                ItemDetails::Runtime {
                    tool,
                    runtime,
                    is_default,
                },
                None,
            ),
            DetailsV1::GlobalPackage { manager } => (ItemDetails::GlobalPackage { manager }, None),
            DetailsV1::Extension { editor } => (ItemDetails::Extension { editor }, None),
            DetailsV1::IdePlugin { ide, folder } => (ItemDetails::IdePlugin { ide, folder }, None),
            DetailsV1::ConfigFile {
                source,
                target,
                content,
                masked_lines,
                revealed_lines,
            } => (
                ItemDetails::ConfigFile {
                    source: PathBuf::from(source),
                    target: target.into_model(),
                },
                Some(StoredConfig {
                    content,
                    masked: masked_lines.into_iter().map(|m| (m.line, m.name)).collect(),
                    revealed: revealed_lines,
                }),
            ),
            DetailsV1::ShellPlugin { url, target } => (
                ItemDetails::ShellPlugin {
                    url,
                    target: target.into_model(),
                },
                None,
            ),
        };
        (
            Item {
                id: self.id,
                provider: self.provider,
                category: self.category,
                name: self.name,
                version: self.version,
                description: self.description,
                catalog_id: self.catalog_id,
                origin: self.origin,
                portability: self.portability,
                details,
                findings: self.findings,
            },
            config,
        )
    }
}
