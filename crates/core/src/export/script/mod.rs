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
        ExportNote::SshKey { file, line, path } => format!("{path} (files/{file} line {line})"),
    }
}
/// Notes listed under `Notes`; SSH keys get their own list.
pub(crate) fn general_notes(notes: &[ExportNote]) -> Vec<&ExportNote> {
    notes
        .iter()
        .filter(|n| !matches!(n, ExportNote::SshKey { .. }))
        .collect()
}
pub(crate) const SSH_KEYS_HEADING: &str =
    "Copy these SSH keys yourself (Repot does not export keys):";
/// Lines under [`SSH_KEYS_HEADING`]: each key path once (first line wins), or the default
/// key names when the config names none. Empty when no SSH file is restored.
pub(crate) fn ssh_key_lines(steps: &[Step], notes: &[ExportNote], target: Os) -> Vec<String> {
    let copies_ssh = steps
        .iter()
        .any(|s| matches!(s, Step::CopyFile { dest, .. } if dest.is_private()));
    let mut seen = std::collections::HashSet::new();
    let mut lines: Vec<String> = notes
        .iter()
        .filter_map(|n| match n {
            ExportNote::SshKey { path, .. } if seen.insert(path) => {
                Some(format!("  - {}", note_text(n)))
            }
            _ => None,
        })
        .collect();
    if lines.is_empty() {
        if !copies_ssh {
            return lines;
        }
        lines.push("  - the keys you use (default names: id_ed25519, id_ecdsa, id_rsa)".into());
    }
    lines.push(if target == Os::Windows {
        "  Copy each key and its .pub file from your old machine into %USERPROFILE%\\.ssh.".into()
    } else {
        "  Copy each key and its .pub file from your old machine into ~/.ssh, then run chmod 600 on the private keys.".into()
    });
    lines
}
impl Step {
    pub fn mapped_origin(&self) -> Option<String> {
        match self {
            Self::InstallPackage {
                mapped_from: Some(from),
                ..
            } => Some(format!(
                "{}: {}",
                package_cmd(from.manager).trim_end_matches("-get"),
                from.name
            )),
            _ => None,
        }
    }
    pub fn label(&self, target: Os) -> String {
        match self {
            Self::AddTap { manager, name, .. } => format!(
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
                ..
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
                components,
                targets,
                ..
            } => {
                let mut label = format!("{} install {runtime} {version}", runtime_name(*tool));
                let extras: Vec<String> = [("components", components), ("targets", targets)]
                    .into_iter()
                    .filter(|(_, names)| !names.is_empty())
                    .map(|(kind, names)| format!("{kind}: {}", names.join(", ")))
                    .collect();
                if !extras.is_empty() {
                    label.push_str(&format!(" ({})", extras.join("; ")));
                }
                label
            }
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
            Self::InstallIdePlugin { ide, id, .. } => {
                format!("{} installPlugins {id}", ide.command())
            }
            Self::CopyFile { dest, .. } => format!("copy {}", display_path(dest, target)),
            Self::GitClone { url, .. } => format!("git clone {url}"),
        }
    }
}
