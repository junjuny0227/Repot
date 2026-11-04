use crate::os::{LinuxFamily, Os, Shell};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Destination OS and its chosen package manager. Commands validate before use.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub os: Os,
    pub packages: Option<PackageManager>,
}
impl Target {
    pub fn default_for(os: Os) -> Self {
        let packages = match os {
            Os::MacOs => Some(PackageManager::Brew),
            Os::Windows => Some(PackageManager::Winget),
            Os::Linux(LinuxFamily::Debian) => Some(PackageManager::Apt),
            Os::Linux(LinuxFamily::Fedora) => Some(PackageManager::Dnf),
            Os::Linux(LinuxFamily::Arch) => Some(PackageManager::Pacman),
            Os::Linux(LinuxFamily::Other) => None,
        };
        Self { os, packages }
    }
    pub fn validate(self) -> Result<(), &'static str> {
        if self == Self::default_for(self.os)
            || (self.os == Os::Windows
                && matches!(
                    self.packages,
                    Some(PackageManager::Winget | PackageManager::Scoop | PackageManager::Choco)
                ))
        {
            Ok(())
        } else {
            Err("invalid package manager for destination OS")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    pub items: Vec<String>,
    pub reveal_secret_lines: Vec<(String, u32)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub provider: String,
    pub category: Category,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    /// Id of the built-in catalog entry the description came from; the UI translates by it.
    pub catalog_id: Option<String>,
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
    Package,
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
    /// A JetBrains IDE plugin; `folder` is its directory or jar name under `plugins`.
    IdePlugin {
        ide: JetBrainsIde,
        folder: String,
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
/// JetBrains IDEs by their settings-folder prefix (plan 14).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum JetBrainsIde {
    IntelliJIdea,
    IdeaIc,
    WebStorm,
    PyCharm,
    PyCharmCe,
    GoLand,
    RustRover,
    CLion,
    PhpStorm,
    RubyMine,
    DataGrip,
    Rider,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum TargetPath {
    Home(PathBuf),
    EditorUser {
        editor: Editor,
        rel: PathBuf,
    },
    PowerShellProfile,
    /// Inside a JetBrains IDE's settings folder. The restore script picks the newest
    /// folder of that IDE on the destination, or `<prefix><version>` if there is none.
    JetBrains {
        ide: JetBrainsIde,
        version: String,
        rel: PathBuf,
    },
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
