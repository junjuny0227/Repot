use crate::{
    model::{Editor, TargetPath},
    os::{LinuxFamily, Os},
};
use std::path::Path;

pub fn bash_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
pub fn ps_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
fn relative(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
fn app(editor: Editor) -> &'static str {
    match editor {
        Editor::VsCode => "Code",
        Editor::Cursor => "Cursor",
    }
}
fn editor_base(editor: Editor, os: Os) -> String {
    let app = app(editor);
    if os == Os::MacOs {
        format!("Library/Application Support/{app}/User/")
    } else {
        format!(".config/{app}/User/")
    }
}
pub fn bash_path(target: &TargetPath, os: Os) -> String {
    match target {
        TargetPath::Home(rel) => format!("\"$HOME/\"{}", bash_quote(&relative(rel))),
        TargetPath::EditorUser { editor, rel } => format!(
            "\"$HOME/{}\"{}",
            editor_base(*editor, os),
            bash_quote(&relative(rel))
        ),
        TargetPath::PowerShellProfile => {
            "\"$HOME/.config/powershell/Microsoft.PowerShell_profile.ps1\"".into()
        }
    }
}
pub fn ps_path(target: &TargetPath) -> String {
    let (base, rel) = match target {
        TargetPath::Home(rel) => ("$HOME", relative(rel)),
        TargetPath::EditorUser { editor, rel } => (
            "$env:APPDATA",
            format!("{}/User/{}", app(*editor), relative(rel)),
        ),
        TargetPath::PowerShellProfile => (
            "([Environment]::GetFolderPath('MyDocuments'))",
            "PowerShell/Microsoft.PowerShell_profile.ps1".into(),
        ),
    };
    format!("(Join-Path {base} {})", ps_quote(&rel.replace('/', "\\")))
}
pub fn display_path(target: &TargetPath, os: Os) -> String {
    if os == Os::Windows {
        return match target {
            TargetPath::Home(rel) => format!("%USERPROFILE%\\{}", relative(rel).replace('/', "\\")),
            TargetPath::EditorUser { editor, rel } => format!(
                "%APPDATA%\\{}\\User\\{}",
                app(*editor),
                relative(rel).replace('/', "\\")
            ),
            TargetPath::PowerShellProfile => {
                "Documents\\PowerShell\\Microsoft.PowerShell_profile.ps1".into()
            }
        };
    }
    match target {
        TargetPath::Home(rel) => format!("~/{}", relative(rel)),
        TargetPath::EditorUser { editor, rel } => {
            format!("~/{}{}", editor_base(*editor, os), relative(rel))
        }
        TargetPath::PowerShellProfile => {
            "~/.config/powershell/Microsoft.PowerShell_profile.ps1".into()
        }
    }
}
pub fn payload_path(target: &TargetPath) -> String {
    match target {
        TargetPath::Home(rel) => format!("home/{}", relative(rel)),
        TargetPath::EditorUser { editor, rel } => format!(
            "{}/{}",
            match editor {
                Editor::VsCode => "vscode",
                Editor::Cursor => "cursor",
            },
            relative(rel)
        ),
        TargetPath::PowerShellProfile => "powershell/Microsoft.PowerShell_profile.ps1".into(),
    }
}
pub fn os_label(os: Os) -> &'static str {
    match os {
        Os::MacOs => "macOS",
        Os::Windows => "Windows",
        Os::Linux(LinuxFamily::Debian) => "Linux (Debian/Ubuntu)",
        Os::Linux(LinuxFamily::Fedora) => "Linux (Fedora)",
        Os::Linux(LinuxFamily::Arch) => "Linux (Arch)",
        Os::Linux(LinuxFamily::Other) => "Linux",
    }
}
