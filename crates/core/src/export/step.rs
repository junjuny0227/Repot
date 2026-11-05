use crate::{availability::counterpart, model::*, os::Os};

#[derive(Debug, Clone, PartialEq)]
pub struct MappedFrom {
    pub manager: PackageManager,
    pub kind: PackageKind,
    pub name: String,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// `url` is set only for a Scoop bucket that Scoop cannot add by name.
    AddTap {
        manager: PackageManager,
        name: String,
        url: Option<String>,
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
        /// rustup only: components and targets the toolchain must have.
        components: Vec<String>,
        targets: Vec<String>,
    },
    InstallGlobal {
        manager: GlobalManager,
        name: String,
    },
    InstallExtension {
        editor: Editor,
        id: String,
    },
    /// `<ide command> installPlugins <id>`; already there when `folder` exists in the
    /// IDE's newest plugins folder on the destination.
    InstallIdePlugin {
        ide: JetBrainsIde,
        id: String,
        folder: String,
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
            Self::InstallExtension { .. } | Self::InstallIdePlugin { .. } => Section::Extensions,
            Self::CopyFile { .. } | Self::GitClone { .. } => Section::Files,
        }
    }
}
pub fn steps_for(item: &Item, target: Os) -> Vec<Step> {
    let step = match &item.details {
        ItemDetails::Package {
            manager,
            kind: PackageKind::Tap,
            url,
        } => Step::AddTap {
            manager: *manager,
            name: item.name.clone(),
            url: url.clone(),
        },
        ItemDetails::Package { manager, kind, .. } => Step::InstallPackage {
            manager: *manager,
            kind: *kind,
            name: item.name.clone(),
            mapped_from: None,
        },
        ItemDetails::Runtime {
            tool,
            runtime,
            is_default,
            components,
            targets,
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
                components: components.clone(),
                targets: targets.clone(),
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
        ItemDetails::IdePlugin { ide, folder } => Step::InstallIdePlugin {
            ide: *ide,
            id: item.name.clone(),
            folder: folder.clone(),
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
                ..
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
                // The package map only uses Scoop's known buckets.
                steps.push(Step::AddTap {
                    manager,
                    name: bucket.into(),
                    url: None,
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
/// A Scoop bucket source: `https://…` or `git@host:path`, made only of safe characters.
/// No `@` after the prefix, so credentials (`https://token@host/…`) never reach the script.
pub fn is_safe_bucket_url(url: &str) -> bool {
    ["https://", "git@"].iter().any(|prefix| {
        url.strip_prefix(prefix)
            .is_some_and(|rest| !rest.is_empty() && !rest.contains('@'))
    }) && is_safe_value(url)
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
        name if JetBrainsIde::ALL.iter().any(|ide| ide.product() == name) => {
            crate::jetbrains::INSTALL_URL
        }
        _ => "",
    }
}
