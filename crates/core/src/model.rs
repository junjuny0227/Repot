use crate::os::Shell;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub provider: String,
    pub category: Category,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub origin: Origin,
    pub portability: Portability,
    pub details: ItemDetails,
    pub findings: Vec<Finding>,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Category {
    PackageManager,
    Runtime,
    GlobalPackage,
    Editor,
    Git,
    Shell,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Origin {
    Command(String),
    Path(PathBuf),
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Portability {
    Any,
    SameOs,
    SameShell(Shell),
    Mapped(RuntimeTool),
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ItemDetails {
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
    ConfigFile {
        source: PathBuf,
        target: TargetPath,
    },
    ShellPlugin {
        url: String,
        target: TargetPath,
    },
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum PackageManager {
    Brew,
    Winget,
    Scoop,
    Choco,
    Apt,
    Dnf,
    Pacman,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum PackageKind {
    Formula,
    Cask,
    Tap,
    Package,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeTool {
    Nvm,
    NvmWindows,
    Fnm,
    Volta,
    Pyenv,
    PyenvWin,
    Rustup,
    Sdkman,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum GlobalManager {
    Npm,
    Pnpm,
    Pipx,
    Cargo,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Editor {
    VsCode,
    Cursor,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum TargetPath {
    Home(PathBuf),
    EditorUser { editor: Editor, rel: PathBuf },
    PowerShellProfile,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub line: Option<u32>,
    pub kind: FindingKind,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum FindingKind {
    Secret { name: Option<String> },
    OsSpecificLine { key: String },
    Duplicate { other_provider: String },
    Unreadable,
}
