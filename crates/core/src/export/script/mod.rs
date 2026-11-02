pub mod bash;
pub mod powershell;

use super::{
    paths::display_path,
    payload::{ExportNote, MaskedLine},
    step::{Section, Step},
};
use crate::{model::*, os::Os};

pub struct ScriptHeader {
    pub repot_version: String,
    pub source: Os,
    pub target: Os,
    pub generated_at: String,
}
pub struct ScriptPlan {
    pub header: ScriptHeader,
    pub steps: Vec<Step>,
    pub masked: Vec<MaskedLine>,
    pub notes: Vec<ExportNote>,
}

pub(crate) fn package_cmd(m: PackageManager) -> &'static str {
    match m {
        PackageManager::Brew => "brew",
        PackageManager::Winget => "winget",
        PackageManager::Scoop => "scoop",
        PackageManager::Choco => "choco",
        PackageManager::Apt => "apt-get",
        PackageManager::Dnf => "dnf",
        PackageManager::Pacman => "pacman",
    }
}
pub(crate) fn runtime_name(t: RuntimeTool) -> &'static str {
    match t {
        RuntimeTool::Nvm => "nvm",
        RuntimeTool::NvmWindows => "nvm-windows",
        RuntimeTool::Fnm => "fnm",
        RuntimeTool::Volta => "volta",
        RuntimeTool::Pyenv => "pyenv",
        RuntimeTool::PyenvWin => "pyenv-win",
        RuntimeTool::Rustup => "rustup",
        RuntimeTool::Sdkman => "sdkman",
    }
}
pub(crate) fn runtime_cmd(t: RuntimeTool) -> &'static str {
    match t {
        RuntimeTool::NvmWindows => "nvm",
        RuntimeTool::PyenvWin => "pyenv",
        RuntimeTool::Sdkman => "sdk",
        _ => runtime_name(t),
    }
}
pub(crate) fn global_cmd(m: GlobalManager) -> &'static str {
    match m {
        GlobalManager::Npm => "npm",
        GlobalManager::Pnpm => "pnpm",
        GlobalManager::Pipx => "pipx",
        GlobalManager::Cargo => "cargo",
    }
}
pub(crate) fn editor_cmd(e: Editor) -> &'static str {
    match e {
        Editor::VsCode => "code",
        Editor::Cursor => "cursor",
    }
}
pub(crate) fn section_name(s: Section) -> &'static str {
    match s {
        Section::Packages => "Packages",
        Section::Runtimes => "Runtimes",
        Section::GlobalPackages => "Global packages",
        Section::Extensions => "Extensions",
        Section::Files => "Files",
    }
}
pub(crate) fn masked_text(m: &MaskedLine) -> String {
    format!(
        "files/{} line {} ({})",
        m.file,
        m.line,
        m.name.as_deref().unwrap_or("value")
    )
}
pub(crate) fn note_text(n: &ExportNote) -> String {
    match n {
        ExportNote::CommentedOsLine { file, line, key } => {
            format!("files/{file} line {line}: {key} was commented out because it is OS-specific.")
        }
        ExportNote::KeybindingsMayDiffer { file } => {
            format!("files/{file}: key bindings may use Cmd vs Ctrl differently on this OS.")
        }
    }
}
impl Step {
    pub fn label(&self, target: Os) -> String {
        match self {
            Self::AddTap { manager, name } => format!(
                "{} {} {name}",
                package_cmd(*manager),
                if *manager == PackageManager::Scoop {
                    "bucket add"
                } else {
                    "tap"
                }
            ),
            Self::InstallPackage {
                manager,
                kind,
                name,
            } => format!(
                "{} install {}{name}",
                package_cmd(*manager),
                if *manager == PackageManager::Brew && *kind == PackageKind::Cask {
                    "--cask "
                } else {
                    ""
                }
            ),
            Self::InstallRuntime {
                tool,
                runtime,
                version,
                ..
            } => format!("{} install {runtime} {version}", runtime_name(*tool)),
            Self::InstallGlobal { manager, name } => format!(
                "{} install {}{name}",
                global_cmd(*manager),
                if matches!(manager, GlobalManager::Npm | GlobalManager::Pnpm) {
                    "-g "
                } else {
                    ""
                }
            ),
            Self::InstallExtension { editor, id } => {
                format!("{} --install-extension {id}", editor_cmd(*editor))
            }
            Self::CopyFile { dest, .. } => format!("copy {}", display_path(dest, target)),
            Self::GitClone { url, .. } => format!("git clone {url}"),
        }
    }
}
