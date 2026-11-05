//! English words for core's structured results. Core makes no sentences (spec §5).

use repot_core::availability::UnavailableReason;
use repot_core::export::ExportWarning;
use repot_core::os::{LinuxFamily, Os, Shell};
use repot_core::scan::ProviderStatus;
use repot_core::{Category, PackageManager, RuntimeTool};

pub fn os(os: Os) -> String {
    match os {
        Os::MacOs => "macOS".into(),
        Os::Windows => "Windows".into(),
        Os::Linux(family) => format!(
            "Linux ({})",
            match family {
                LinuxFamily::Debian => "Debian/Ubuntu",
                LinuxFamily::Fedora => "Fedora",
                LinuxFamily::Arch => "Arch",
                LinuxFamily::Other => "other",
            }
        ),
    }
}

pub fn category(c: Category) -> &'static str {
    match c {
        Category::PackageManager => "Packages",
        Category::Runtime => "Runtimes",
        Category::GlobalPackage => "Global packages",
        Category::Editor => "Editors",
        Category::Git => "Git",
        Category::Shell => "Shell",
        Category::Ssh => "SSH",
    }
}

pub fn manager(m: PackageManager) -> &'static str {
    match m {
        PackageManager::Brew => "Homebrew",
        PackageManager::Winget => "winget",
        PackageManager::Scoop => "Scoop",
        PackageManager::Choco => "Chocolatey",
        PackageManager::Apt => "apt",
        PackageManager::Dnf => "dnf",
        PackageManager::Pacman => "pacman",
    }
}

fn shell(s: Shell) -> &'static str {
    match s {
        Shell::Zsh => "zsh",
        Shell::Bash => "bash",
        Shell::Fish => "fish",
        Shell::PowerShell => "PowerShell",
    }
}

fn runtime_tool(t: RuntimeTool) -> &'static str {
    match t {
        RuntimeTool::Nvm => "nvm",
        RuntimeTool::NvmWindows => "nvm-windows",
        RuntimeTool::Fnm => "fnm",
        RuntimeTool::Volta => "Volta",
        RuntimeTool::Pyenv => "pyenv",
        RuntimeTool::PyenvWin => "pyenv-win",
        RuntimeTool::Rustup => "rustup",
        RuntimeTool::Sdkman => "SDKMAN!",
    }
}

pub fn reason(r: &UnavailableReason) -> String {
    match r {
        UnavailableReason::Unreadable => "the file could not be read".into(),
        UnavailableReason::DifferentOs { source } => format!("only works on {}", os(*source)),
        UnavailableReason::ShellUnsupported(s) => {
            format!("{} is not available on the destination OS", shell(*s))
        }
        UnavailableReason::NoCounterpart(t) => {
            format!(
                "{} has no counterpart on the destination OS",
                runtime_tool(*t)
            )
        }
        UnavailableReason::NoPackageManager => {
            "the destination OS has no supported package manager".into()
        }
        UnavailableReason::NotInPackageMap { manager: m } => {
            format!("Repot's package map has no {} name for it yet", manager(*m))
        }
        UnavailableReason::NoPackageInManager { manager: m } => {
            format!("it is not available in {}", manager(*m))
        }
    }
}

pub fn warning(w: &ExportWarning, item: &str) -> String {
    match w {
        ExportWarning::Unavailable { reason: r, .. } => {
            format!("skipped {item}: {}", reason(r))
        }
        ExportWarning::InvalidValue { .. } => {
            format!("skipped {item}: it contains characters that are unsafe in a script")
        }
        ExportWarning::DuplicateTarget { .. } => {
            format!("skipped {item}: another item writes to the same file")
        }
        ExportWarning::ReadFailed { .. } => format!("skipped {item}: the file could not be read"),
    }
}

pub fn provider_status(s: &ProviderStatus) -> String {
    match s {
        ProviderStatus::Running => "running".into(),
        ProviderStatus::Ok { count } => format!("{count} found"),
        ProviderStatus::NotDetected => "not installed".into(),
        ProviderStatus::Failed { message, .. } => format!("failed: {message}"),
        ProviderStatus::TimedOut => "timed out".into(),
    }
}
