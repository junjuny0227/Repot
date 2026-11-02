use repot_core::{
    export::{
        payload::{ExportNote, MaskedLine},
        script::{ScriptHeader, ScriptPlan, bash::render_bash, powershell::render_powershell},
        step::{Step, sort_steps},
    },
    model::*,
    os::{LinuxFamily, Os},
};
use std::{error::Error, path::PathBuf};

fn sample_plan(target: Os) -> ScriptPlan {
    let mut steps = vec![];
    let packages: &[PackageManager] = match target {
        Os::MacOs => &[PackageManager::Brew],
        Os::Windows => &[
            PackageManager::Winget,
            PackageManager::Scoop,
            PackageManager::Choco,
        ],
        Os::Linux(_) => &[
            PackageManager::Apt,
            PackageManager::Dnf,
            PackageManager::Pacman,
        ],
    };
    for &manager in packages {
        if matches!(manager, PackageManager::Brew | PackageManager::Scoop) {
            steps.push(Step::AddTap {
                manager,
                name: if manager == PackageManager::Brew {
                    "homebrew/cask"
                } else {
                    "extras"
                }
                .into(),
            });
        }
        steps.push(Step::InstallPackage {
            manager,
            kind: if manager == PackageManager::Brew {
                PackageKind::Formula
            } else {
                PackageKind::Package
            },
            name: "ripgrep".into(),
        });
    }
    if target == Os::MacOs {
        steps.push(Step::InstallPackage {
            manager: PackageManager::Brew,
            kind: PackageKind::Cask,
            name: "visual-studio-code".into(),
        });
    }
    let runtimes: &[RuntimeTool] = if target == Os::Windows {
        &[
            RuntimeTool::NvmWindows,
            RuntimeTool::Fnm,
            RuntimeTool::Volta,
            RuntimeTool::PyenvWin,
            RuntimeTool::Rustup,
        ]
    } else {
        &[
            RuntimeTool::Nvm,
            RuntimeTool::Fnm,
            RuntimeTool::Volta,
            RuntimeTool::Pyenv,
            RuntimeTool::Rustup,
            RuntimeTool::Sdkman,
        ]
    };
    for &tool in runtimes {
        let (runtime, version) = match tool {
            RuntimeTool::Pyenv | RuntimeTool::PyenvWin => ("python", "3.12.7"),
            RuntimeTool::Rustup => ("rust", "stable"),
            RuntimeTool::Sdkman => ("java", "21.0.5-tem"),
            _ => ("node", "22.11.0"),
        };
        steps.push(Step::InstallRuntime {
            tool,
            runtime: runtime.into(),
            version: version.into(),
            set_default: true,
        });
    }
    for (manager, name) in [
        (GlobalManager::Npm, "cowsay"),
        (GlobalManager::Pnpm, "typescript"),
        (GlobalManager::Pipx, "ruff"),
        (GlobalManager::Cargo, "bat"),
    ] {
        steps.push(Step::InstallGlobal {
            manager,
            name: name.into(),
        });
    }
    for editor in [Editor::VsCode, Editor::Cursor] {
        steps.push(Step::InstallExtension {
            editor,
            id: "dbaeumer.vscode-eslint".into(),
        });
        steps.push(Step::CopyFile {
            src: "editor/settings.json".into(),
            dest: TargetPath::EditorUser {
                editor,
                rel: "settings.json".into(),
            },
        });
    }
    steps.push(Step::CopyFile {
        src: "home/.repot-sample".into(),
        dest: TargetPath::Home(".repot-sample".into()),
    });
    steps.push(Step::CopyFile {
        src: "powershell/profile.ps1".into(),
        dest: TargetPath::PowerShellProfile,
    });
    if target != Os::Windows {
        steps.push(Step::GitClone {
            url: "https://github.com/zsh-users/zsh-autosuggestions.git".into(),
            dest: TargetPath::Home(".oh-my-zsh/custom/plugins/zsh-autosuggestions".into()),
        });
    }
    sort_steps(&mut steps);
    ScriptPlan {
        header: ScriptHeader {
            repot_version: env!("CARGO_PKG_VERSION").into(),
            source: target,
            target,
            generated_at: "2026-11-02T00:00:00Z".into(),
        },
        steps,
        masked: vec![MaskedLine {
            file: "home/.repot-sample".into(),
            line: 1,
            name: Some("DEMO_TOKEN".into()),
        }],
        notes: vec![ExportNote::KeybindingsMayDiffer {
            file: "editor/keybindings.json".into(),
        }],
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let out = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("usage: render_samples <out>")?,
    );
    std::fs::create_dir_all(&out)?;
    for (name, target) in [
        ("restore-macos.sh", Os::MacOs),
        ("restore-linux.sh", Os::Linux(LinuxFamily::Debian)),
        ("restore.ps1", Os::Windows),
    ] {
        let plan = sample_plan(target);
        let text = if target == Os::Windows {
            render_powershell(&plan)
        } else {
            render_bash(&plan)
        };
        std::fs::write(out.join(name), text)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_cover_all_steps_and_supported_tools() {
        let plans = [
            sample_plan(Os::MacOs),
            sample_plan(Os::Linux(LinuxFamily::Debian)),
            sample_plan(Os::Windows),
        ];
        let steps: Vec<_> = plans.iter().flat_map(|p| &p.steps).collect();
        for manager in [
            PackageManager::Brew,
            PackageManager::Winget,
            PackageManager::Scoop,
            PackageManager::Choco,
            PackageManager::Apt,
            PackageManager::Dnf,
            PackageManager::Pacman,
        ] {
            assert!(
                steps
                    .iter()
                    .any(|s| matches!(s, Step::InstallPackage { manager: m, .. } if *m == manager))
            );
        }
        for tool in [
            RuntimeTool::Nvm,
            RuntimeTool::NvmWindows,
            RuntimeTool::Fnm,
            RuntimeTool::Volta,
            RuntimeTool::Pyenv,
            RuntimeTool::PyenvWin,
            RuntimeTool::Rustup,
            RuntimeTool::Sdkman,
        ] {
            assert!(steps.iter().any(|s| matches!(s, Step::InstallRuntime { tool: t, set_default: true, .. } if *t == tool)));
        }
        for plan in &plans {
            for manager in [
                GlobalManager::Npm,
                GlobalManager::Pnpm,
                GlobalManager::Pipx,
                GlobalManager::Cargo,
            ] {
                assert!(
                    plan.steps.iter().any(
                        |s| matches!(s, Step::InstallGlobal { manager: m, .. } if *m == manager)
                    )
                );
            }
            for editor in [Editor::VsCode, Editor::Cursor] {
                assert!(plan.steps.iter().any(
                    |s| matches!(s, Step::InstallExtension { editor: e, .. } if *e == editor)
                ));
            }
            assert!(
                plan.steps
                    .iter()
                    .any(|s| matches!(s, Step::CopyFile { .. }))
            );
            assert_eq!(plan.masked.len(), 1);
            assert_eq!(plan.notes.len(), 1);
        }
        assert!(steps.iter().any(|s| matches!(s, Step::AddTap { .. })));
        assert!(steps.iter().any(|s| matches!(s, Step::GitClone { .. })));
        assert!(!plans[2].steps.iter().any(|s| matches!(
            s,
            Step::GitClone { .. }
                | Step::InstallRuntime {
                    tool: RuntimeTool::Nvm | RuntimeTool::Pyenv | RuntimeTool::Sdkman,
                    ..
                }
        )));
    }
}
