use crate::{availability::counterpart, model::*, os::Os};

#[derive(Debug, Clone, PartialEq)]
pub struct MappedFrom {
    pub manager: PackageManager,
    pub kind: PackageKind,
    pub name: String,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    AddTap {
        manager: PackageManager,
        name: String,
    },
    InstallPackage {
        manager: PackageManager,
        kind: PackageKind,
        name: String,
        mapped_from: Option<MappedFrom>,
    },
    InstallRuntime {
        tool: RuntimeTool,
        runtime: String,
        version: String,
        set_default: bool,
    },
    InstallGlobal {
        manager: GlobalManager,
        name: String,
    },
    InstallExtension {
        editor: Editor,
        id: String,
    },
    CopyFile {
        src: String,
        dest: TargetPath,
    },
    GitClone {
        url: String,
        dest: TargetPath,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section {
    Packages,
    Runtimes,
    GlobalPackages,
    Extensions,
    Files,
}
impl Step {
    pub fn section(&self) -> Section {
        match self {
            Self::AddTap { .. } | Self::InstallPackage { .. } => Section::Packages,
            Self::InstallRuntime { .. } => Section::Runtimes,
            Self::InstallGlobal { .. } => Section::GlobalPackages,
            Self::InstallExtension { .. } => Section::Extensions,
            Self::CopyFile { .. } | Self::GitClone { .. } => Section::Files,
        }
    }
}
pub fn steps_for(item: &Item, target: Os) -> Vec<Step> {
    let step = match &item.details {
        ItemDetails::Package {
            manager,
            kind: PackageKind::Tap,
        } => Step::AddTap {
            manager: *manager,
            name: item.name.clone(),
        },
        ItemDetails::Package { manager, kind } => Step::InstallPackage {
            manager: *manager,
            kind: *kind,
            name: item.name.clone(),
            mapped_from: None,
        },
        ItemDetails::Runtime {
            tool,
            runtime,
            is_default,
        } => {
            let (Some(tool), Some(version)) = (counterpart(*tool, target), item.version.as_ref())
            else {
                return vec![];
            };
            Step::InstallRuntime {
                tool,
                runtime: runtime.clone(),
                version: version.clone(),
                set_default: *is_default,
            }
        }
        ItemDetails::GlobalPackage { manager } => Step::InstallGlobal {
            manager: *manager,
            name: item.name.clone(),
        },
        ItemDetails::Extension { editor } => Step::InstallExtension {
            editor: *editor,
            id: item.name.clone(),
        },
        ItemDetails::ShellPlugin { url, target } => Step::GitClone {
            url: url.clone(),
            dest: target.clone(),
        },
        ItemDetails::ConfigFile { target, .. } => Step::CopyFile {
            src: super::paths::payload_path(target),
            dest: target.clone(),
        },
    };
    vec![step]
}
/// Apply availability before generating steps; unknown or absent mappings never leak through.
pub fn steps_for_target(item: &Item, source: Os, target: Target) -> Vec<Step> {
    use crate::{
        availability::{Availability, availability},
        package_map::{Lookup, embedded},
    };
    match availability(item, source, target) {
        Availability::Unavailable(_) => vec![],
        Availability::Available => steps_for(item, target.os),
        Availability::Mapped { manager, name } => {
            let ItemDetails::Package {
                manager: original,
                kind: original_kind,
            } = item.details
            else {
                return vec![];
            };
            let Lookup::Found { kind, .. } =
                embedded().lookup(original, original_kind, &item.name, manager)
            else {
                return vec![];
            };
            let mut steps = vec![];
            if manager == PackageManager::Scoop {
                let bucket = name.split_once('/').map_or("main", |(bucket, _)| bucket);
                steps.push(Step::AddTap {
                    manager,
                    name: bucket.into(),
                });
            }
            steps.push(Step::InstallPackage {
                manager,
                kind,
                name,
                mapped_from: Some(MappedFrom {
                    manager: original,
                    kind: original_kind,
                    name: item.name.clone(),
                }),
            });
            steps
        }
    }
}
/// Stable ordering preserves discovery order within each section.
pub fn sort_steps(steps: &mut [Step]) {
    steps.sort_by_key(|s| (s.section(), !matches!(s, Step::AddTap { .. })));
}
pub fn is_safe_value(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"@._+/:-".contains(&b))
}
pub fn guide_url(tool: &str) -> &'static str {
    match tool {
        "brew" => "https://brew.sh",
        "winget" => "https://learn.microsoft.com/windows/package-manager/winget/",
        "scoop" => "https://scoop.sh",
        "choco" => "https://chocolatey.org/install",
        "nvm" => "https://github.com/nvm-sh/nvm",
        "nvm-windows" => "https://github.com/coreybutler/nvm-windows",
        "fnm" => "https://github.com/Schniz/fnm",
        "volta" => "https://volta.sh",
        "pyenv" => "https://github.com/pyenv/pyenv",
        "pyenv-win" => "https://github.com/pyenv-win/pyenv-win",
        "rustup" | "cargo" => "https://rustup.rs",
        "sdkman" => "https://sdkman.io",
        "npm" | "pnpm" => "https://nodejs.org",
        "pipx" => "https://pipx.pypa.io",
        "code" => "https://code.visualstudio.com/docs/setup/setup-overview",
        "cursor" => "https://cursor.com",
        "git" => "https://git-scm.com",
        "oh-my-zsh" => "https://ohmyz.sh",
        _ => "",
    }
}
