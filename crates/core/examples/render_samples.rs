use repot_core::{
    export::{
        payload::{ExportNote, MaskedLine, prepare_config},
        script::{ScriptHeader, ScriptPlan, bash::render_bash, powershell::render_powershell},
        step::{Step, sort_steps},
    },
    model::*,
    os::{LinuxFamily, Os},
};
use std::{error::Error, path::PathBuf};

/// The plan 16 example `~/.ssh/config` from a Mac. CI runs real OpenSSH on the copies
/// prepared for each destination (`ssh -G` exits 255 on an option it does not know).
const SSH_CONFIG: &str = "Include config.d/*
Include ~/.orbstack/ssh/config

Host *
  AddKeysToAgent yes
  UseKeychain yes
  IdentityFile ~/.ssh/id_ed25519

Host github-work
  HostName github.com
  User git
  IdentityFile ~/.ssh/work_ed25519
  IdentitiesOnly yes

Host bastion
  HostName bastion.example.com
  ControlMaster auto
  ControlPath ~/.ssh/cm-%r@%h:%p
  ControlPersist 10m
  SetEnv DEPLOY_TOKEN=abc123
";

/// The example config as an export from macOS to `target` writes it.
fn ssh_config_for(target: Os) -> String {
    let source = PathBuf::from("/Users/demo/.ssh/config");
    let item = Item {
        id: "ssh:file:.ssh/config".into(),
        provider: "ssh".into(),
        category: Category::Ssh,
        name: ".ssh/config".into(),
        version: None,
        description: None,
        catalog_id: Some("file:ssh-config".into()),
        origin: Origin::Path(source.clone()),
        portability: Portability::Any,
        details: ItemDetails::ConfigFile {
            source,
            target: TargetPath::Home(".ssh/config".into()),
        },
        findings: vec![],
    };
    prepare_config(&item, SSH_CONFIG, &[], Os::MacOs, target).contents
}

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
                url: None,
            });
        }
        if manager == PackageManager::Scoop {
            // A custom bucket is added with its URL.
            steps.push(Step::AddTap {
                manager,
                name: "repot".into(),
                url: Some("https://github.com/junjuny0227/scoop-bucket".into()),
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
            mapped_from: None,
        });
    }
    if target == Os::MacOs {
        steps.push(Step::InstallPackage {
            manager: PackageManager::Brew,
            kind: PackageKind::Cask,
            name: "visual-studio-code".into(),
            mapped_from: None,
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
        // rustup lists components and targets so the linters see its helper and commands.
        let (components, targets) = if tool == RuntimeTool::Rustup {
            (
                vec!["clippy".to_owned(), "rust-src".to_owned()],
                vec!["wasm32-unknown-unknown".to_owned()],
            )
        } else {
            (vec![], vec![])
        };
        steps.push(Step::InstallRuntime {
            tool,
            runtime: runtime.into(),
            version: version.into(),
            set_default: true,
            components,
            targets,
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
    // JetBrains helpers are emitted only with these steps; CI lints them (plan 14).
    steps.push(Step::InstallIdePlugin {
        ide: JetBrainsIde::IntelliJIdea,
        id: "com.github.copilot".into(),
        folder: "github-copilot-intellij".into(),
    });
    steps.push(Step::CopyFile {
        src: "jetbrains/IntelliJIdea/codestyles/Default.xml".into(),
        dest: TargetPath::JetBrains {
            ide: JetBrainsIde::IntelliJIdea,
            version: "2026.1".into(),
            rel: "codestyles/Default.xml".into(),
        },
    });
    steps.push(Step::CopyFile {
        src: "home/.repot-sample".into(),
        dest: TargetPath::Home(".repot-sample".into()),
    });
    steps.push(Step::CopyFile {
        src: "powershell/profile.ps1".into(),
        dest: TargetPath::PowerShellProfile,
    });
    // SSH config files: the linters see `copy_private_file` and the key list (plan 16).
    for rel in [".ssh/config", ".ssh/config.d/work"] {
        steps.push(Step::CopyFile {
            src: format!("home/{rel}"),
            dest: TargetPath::Home(rel.into()),
        });
    }
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
        notes: vec![
            ExportNote::KeybindingsMayDiffer {
                file: "editor/keybindings.json".into(),
            },
            ExportNote::SshKey {
                file: "home/.ssh/config".into(),
                line: 7,
                path: "~/.ssh/id_ed25519".into(),
            },
        ],
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
    for (name, target) in [
        ("ssh-config-macos", Os::MacOs),
        ("ssh-config-linux", Os::Linux(LinuxFamily::Debian)),
        ("ssh-config-windows", Os::Windows),
    ] {
        std::fs::write(out.join(name), ssh_config_for(target))?;
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
            assert!(
                plan.steps
                    .iter()
                    .any(|s| matches!(s, Step::InstallIdePlugin { .. }))
            );
            assert_eq!(plan.masked.len(), 1);
            assert_eq!(plan.notes.len(), 2);
            assert!(
                plan.steps
                    .iter()
                    .any(|s| matches!(s, Step::CopyFile { dest, .. } if dest.is_private()))
            );
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

    #[test]
    fn ssh_samples_comment_out_what_each_destination_rejects() {
        let commented = |text: &str| -> Vec<String> {
            text.lines()
                .filter_map(|l| l.trim_start().strip_prefix("# [repot] "))
                .map(|l| l.split_whitespace().next().unwrap().to_owned())
                .collect()
        };
        let mac = ssh_config_for(Os::MacOs);
        let linux = ssh_config_for(Os::Linux(LinuxFamily::Debian));
        let windows = ssh_config_for(Os::Windows);
        assert!(commented(&mac).is_empty());
        assert_eq!(commented(&linux), ["UseKeychain"]);
        assert_eq!(
            commented(&windows),
            [
                "UseKeychain",
                "ControlMaster",
                "ControlPath",
                "ControlPersist"
            ]
        );
        for text in [&mac, &linux, &windows] {
            // Same line numbers as the source; the host CI resolves is still there.
            assert_eq!(text.lines().count(), SSH_CONFIG.lines().count());
            assert!(text.contains("Host github-work\n  HostName github.com\n"));
            assert!(text.contains("SetEnv DEPLOY_TOKEN=<REDACTED>"));
        }
    }
}
