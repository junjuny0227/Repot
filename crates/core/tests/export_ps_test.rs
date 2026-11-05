use repot_core::{
    export::{
        payload::{ExportNote, MaskedLine},
        script::{ScriptHeader, ScriptPlan, powershell::render_powershell},
        step::Step,
    },
    model::*,
    os::Os,
};
fn windows_plan() -> ScriptPlan {
    ScriptPlan {
        header: ScriptHeader {
            repot_version: "0.1.0".into(),
            source: Os::MacOs,
            target: Os::Windows,
            generated_at: "2026-11-02 10:00 UTC".into(),
        },
        steps: vec![
            Step::AddTap {
                manager: PackageManager::Scoop,
                name: "extras".into(),
                url: None,
            },
            Step::InstallPackage {
                manager: PackageManager::Winget,
                kind: PackageKind::Package,
                name: "Git.Git".into(),
                mapped_from: None,
            },
            Step::InstallPackage {
                manager: PackageManager::Scoop,
                kind: PackageKind::Package,
                name: "extras/vlc".into(),
                mapped_from: None,
            },
            Step::InstallPackage {
                manager: PackageManager::Choco,
                kind: PackageKind::Package,
                name: "7zip".into(),
                mapped_from: None,
            },
            Step::InstallRuntime {
                tool: RuntimeTool::NvmWindows,
                runtime: "node".into(),
                version: "20.11.0".into(),
                set_default: true,
                components: vec![],
                targets: vec![],
            },
            Step::InstallGlobal {
                manager: GlobalManager::Npm,
                name: "typescript".into(),
            },
            Step::InstallExtension {
                editor: Editor::VsCode,
                id: "esbenp.prettier-vscode".into(),
            },
            Step::CopyFile {
                src: "home/.gitconfig".into(),
                dest: TargetPath::Home(".gitconfig".into()),
            },
            Step::CopyFile {
                src: "powershell/Microsoft.PowerShell_profile.ps1".into(),
                dest: TargetPath::PowerShellProfile,
            },
            Step::CopyFile {
                src: "home/.ssh/config".into(),
                dest: TargetPath::Home(".ssh/config".into()),
            },
        ],
        masked: vec![MaskedLine {
            file: "home/.gitconfig".into(),
            line: 9,
            name: None,
        }],
        notes: vec![
            ExportNote::SshKey {
                file: "home/.ssh/config".into(),
                line: 7,
                path: "~/.ssh/id_ed25519".into(),
            },
            ExportNote::CommentedOsLine {
                file: "home/.ssh/config".into(),
                line: 6,
                key: "UseKeychain".into(),
            },
        ],
    }
}
#[test]
fn native_output_is_assigned_before_conditionals() {
    let s = render_powershell(&windows_plan());
    assert!(s.contains("function Test-Tool($Name)"));
    assert!(!s.contains("Test-Have"));
    assert!(!s.contains("if (& "));
    assert!(s.contains("$installed = @((& scoop bucket list 2>$null).Name) -contains 'extras'"));
    assert!(s.contains("if ($installed) { Add-Done"));
}

#[test]
fn custom_scoop_bucket_is_added_with_its_url() {
    let mut plan = windows_plan();
    plan.steps = vec![Step::AddTap {
        manager: PackageManager::Scoop,
        name: "repot".into(),
        url: Some("https://github.com/junjuny0227/scoop-bucket".into()),
    }];
    let s = render_powershell(&plan);
    assert!(s.contains("& scoop bucket add 'repot' 'https://github.com/junjuny0227/scoop-bucket'"));
    assert!(s.contains("$installed = @((& scoop bucket list 2>$null).Name) -contains 'repot'"));
}
#[test]
fn ps_snapshot() {
    insta::assert_snapshot!(render_powershell(&windows_plan()));
}
#[test]
fn package_installs_are_verified_after_a_zero_exit_code() {
    // choco 2.x exits 0 with "installed 0/0 packages" when its feed returns 504.
    let s = render_powershell(&windows_plan());
    assert!(s.contains(
        "else { & choco install '7zip' -y; if ($LASTEXITCODE -eq 0) { $installed = & choco list --exact '7zip' --limit-output 2>$null | Select-String -SimpleMatch '7zip' -Quiet; if ($installed) { Add-Ok"
    ));
    assert!(s.contains("} else { Write-Host '  installer reported success, but the package is not installed'; Add-Fail"));
}
#[test]
fn ps_guards_choco_with_admin_and_quotes_values() {
    let s = render_powershell(&windows_plan());
    assert!(s.contains("if (-not (Test-Admin))"));
    assert!(s.contains("param([switch]$Yes)"));
    let mut plan = windows_plan();
    plan.steps = vec![Step::CopyFile {
        src: "home/it's.json".into(),
        dest: TargetPath::Home("it's.json".into()),
    }];
    assert!(
        render_powershell(&plan)
            .contains("Copy-RepotFile 'home/it''s.json' (Join-Path $HOME 'it''s.json')")
    );
}
#[test]
fn no_admin_guard_without_choco() {
    let mut p = windows_plan();
    p.steps.retain(|s| {
        !matches!(
            s,
            Step::InstallPackage {
                manager: PackageManager::Choco,
                ..
            }
        )
    });
    assert!(!render_powershell(&p).contains("if (-not (Test-Admin))"));
}
#[cfg(windows)]
#[test]
fn ps_copy_backs_up_once_on_windows() {
    let dir = tempfile::tempdir().unwrap();
    let appdata = dir.path().join("appdata");
    std::fs::create_dir_all(appdata.join("Code/User")).unwrap();
    std::fs::write(appdata.join("Code/User/settings.json"), "old").unwrap();
    std::fs::create_dir_all(dir.path().join("files/vscode")).unwrap();
    std::fs::write(dir.path().join("files/vscode/settings.json"), "new").unwrap();
    let mut p = windows_plan();
    p.steps = vec![Step::CopyFile {
        src: "vscode/settings.json".into(),
        dest: TargetPath::EditorUser {
            editor: Editor::VsCode,
            rel: "settings.json".into(),
        },
    }];
    std::fs::write(dir.path().join("restore.ps1"), render_powershell(&p)).unwrap();
    let run = || {
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(dir.path().join("restore.ps1"))
            .arg("-Yes")
            .env("APPDATA", &appdata)
            .output()
            .unwrap()
    };
    let backups = || {
        std::fs::read_dir(appdata.join("Code/User"))
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with("settings.json.repot-backup-")
            })
            .count()
    };
    assert!(run().status.success());
    assert_eq!(
        std::fs::read_to_string(appdata.join("Code/User/settings.json")).unwrap(),
        "new"
    );
    assert_eq!(backups(), 1);
    let second = run();
    assert!(second.status.success());
    let out = String::from_utf8_lossy(&second.stdout);
    assert!(
        out.contains("already done: copy"),
        "stdout:\n{out}\nstderr:\n{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(backups(), 1);
}
#[test]
fn preflight_lists_each_tool_once_before_the_prompt() {
    let s = render_powershell(&windows_plan());
    let preflight = s.find("Write-Host 'Tools this script uses:'").unwrap();
    assert!(preflight < s.find("Continue? [y/N]").unwrap());
    assert_eq!(
        s.matches("Test-Preflight 'winget' 'winget'").count(),
        1,
        "{s}"
    );
}
#[test]
fn steps_that_are_not_attempted_carry_a_reason() {
    let s = render_powershell(&windows_plan());
    assert!(
        s.contains("Add-NotDone 'winget install Git.Git' 'winget not found'"),
        "{s}"
    );
    assert!(s.contains("To finish these, install the missing tools and run this script again:"));
    assert!(!s.contains("Add-Skip"));
}
#[test]
fn runtime_default_only_runs_after_successful_install() {
    let s = render_powershell(&windows_plan());
    assert!(s.contains("if ($LASTEXITCODE -eq 0) { & nvm use '20.11.0' }"));
}

#[test]
fn jetbrains_helpers_appear_only_when_needed() {
    let plain = render_powershell(&windows_plan());
    assert!(!plain.contains("Get-JetBrainsConfig"));
    let mut p = windows_plan();
    p.steps = vec![
        Step::InstallIdePlugin {
            ide: JetBrainsIde::IdeaIc,
            id: "com.github.copilot".into(),
            folder: "github-copilot-intellij".into(),
        },
        Step::CopyFile {
            src: "jetbrains/IdeaIC/keymaps/K.xml".into(),
            dest: TargetPath::JetBrains {
                ide: JetBrainsIde::IdeaIc,
                version: "2026.1".into(),
                rel: "keymaps/K.xml".into(),
            },
        },
    ];
    let s = render_powershell(&p);
    assert!(
        s.contains("$JbCli = Find-JetBrainsCli 'idea' @('IntelliJ IDEA Community Edition*')"),
        "{s}"
    );
    assert!(s.contains("$installed = Test-Path (Join-Path (Get-JetBrainsPluginDir 'IdeaIC') 'github-copilot-intellij')"), "{s}");
    assert!(s.contains("& $JbCli installPlugins 'com.github.copilot'"));
    assert!(
        s.contains("(Join-Path (Get-JetBrainsConfig 'IdeaIC' '2026.1') 'keymaps\\K.xml')"),
        "{s}"
    );
    assert!(
        s.contains(
            "Add-NotDone 'idea installPlugins com.github.copilot' 'IntelliJ IDEA CE not found'"
        ),
        "{s}"
    );
}

fn rustup_step(version: &str, components: &[&str], targets: &[&str]) -> Step {
    Step::InstallRuntime {
        tool: RuntimeTool::Rustup,
        runtime: "rust".into(),
        version: version.into(),
        set_default: false,
        components: components.iter().map(|s| (*s).to_owned()).collect(),
        targets: targets.iter().map(|s| (*s).to_owned()).collect(),
    }
}

#[test]
fn rustup_steps_check_and_add_components_and_targets() {
    let plain = render_powershell(&windows_plan());
    assert!(!plain.contains("Test-RustupToolchain"));
    let mut p = windows_plan();
    p.steps = vec![
        rustup_step(
            "stable",
            &["clippy", "rust-src"],
            &["wasm32-unknown-unknown"],
        ),
        rustup_step("1.80.0", &[], &[]),
    ];
    let s = render_powershell(&p);
    assert!(
        s.contains("function Test-RustupToolchain($Toolchain, $Components, $Targets) {"),
        "{s}"
    );
    assert!(
        s.contains("  $installed = Test-RustupToolchain 'stable' @('clippy', 'rust-src') @('wasm32-unknown-unknown')\n  if ($installed) { Add-Done 'rustup install rust stable (components: clippy, rust-src; targets: wasm32-unknown-unknown)' }"),
        "{s}"
    );
    assert!(
        s.contains("  else { $hasToolchain = Test-RustupToolchain 'stable' @() @(); if (-not $hasToolchain) { & rustup toolchain install 'stable' }; if ($hasToolchain -or $LASTEXITCODE -eq 0) { & rustup component add --toolchain 'stable' 'clippy' 'rust-src' }; if ($LASTEXITCODE -eq 0) { & rustup target add --toolchain 'stable' 'wasm32-unknown-unknown' }; if ($LASTEXITCODE -eq 0) { Add-Ok"),
        "{s}"
    );
    assert!(
        s.contains("  $installed = Test-RustupToolchain '1.80.0' @() @()\n  if ($installed) { Add-Done 'rustup install rust 1.80.0' }\n  else { & rustup toolchain install '1.80.0'; if ($LASTEXITCODE -eq 0) { Add-Ok"),
        "{s}"
    );
}

/// Same scenario as the Bash test: a stand-in rustup.ps1 on PATH with state in files.
#[cfg(windows)]
#[test]
fn rustup_adds_missing_components_without_reinstalling_the_toolchain_on_windows() {
    let dir = tempfile::tempdir().unwrap();
    let bin = dir.path().join("bin");
    let state = dir.path().join("rustup");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    let host = "x86_64-pc-windows-msvc";
    std::fs::write(
        state.join("toolchains"),
        format!("stable-{host} (default)\nnightly-2024-11-01-{host}\n"),
    )
    .unwrap();
    std::fs::write(
        state.join("components-stable"),
        format!("cargo-{host}\nclippy-{host}\nrust-std-{host}\nrustc-{host}\n"),
    )
    .unwrap();
    std::fs::write(state.join("targets-stable"), format!("{host}\n")).unwrap();
    let script = format!(
        r#"$s = '{state}'; $h = '{host}'; $a = $args
Add-Content -Path "$s\log" -Value ($a -join ' ')
function Short($n) {{ if ($n.EndsWith("-$h")) {{ $n.Substring(0, $n.Length - $h.Length - 1) }} else {{ $n }} }}
switch ("$($a[0]) $($a[1])") {{
  'toolchain list' {{ Get-Content "$s\toolchains" }}
  'toolchain install' {{
    $t = $a[2]
    Add-Content -Path "$s\toolchains" -Value "$t-$h"
    Set-Content -Path "$s\components-$t" -Value @("cargo-$h", "rustc-$h", "rust-std-$h")
    Set-Content -Path "$s\targets-$t" -Value $h
  }}
  'component list' {{ Get-Content "$s\components-$(Short $a[4])" }}
  'target list' {{ Get-Content "$s\targets-$(Short $a[4])" }}
  'component add' {{
    $t = $a[3]
    foreach ($c in $a[4..($a.Count - 1)]) {{ if ($c -eq 'rust-src') {{ Add-Content "$s\components-$t" $c }} else {{ Add-Content "$s\components-$t" "$c-$h" }} }}
  }}
  'target add' {{ $t = $a[3]; foreach ($x in $a[4..($a.Count - 1)]) {{ Add-Content "$s\targets-$t" $x }} }}
  default {{ exit 1 }}
}}
exit 0
"#,
        state = state.display()
    );
    std::fs::write(bin.join("rustup.ps1"), script).unwrap();
    let mut p = windows_plan();
    p.steps = vec![
        rustup_step(
            "stable",
            &["clippy", "rust-src"],
            &["wasm32-unknown-unknown"],
        ),
        rustup_step("1.80.0", &["clippy"], &[]),
        rustup_step("nightly", &[], &[]),
    ];
    std::fs::write(dir.path().join("restore.ps1"), render_powershell(&p)).unwrap();
    // Only the stand-in and Windows itself: a real rustup on the runner must not answer.
    let system = std::env::var("SystemRoot").unwrap();
    let path = format!(
        "{};{system}\\System32;{system}\\System32\\WindowsPowerShell\\v1.0",
        bin.display()
    );
    let run = || {
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(dir.path().join("restore.ps1"))
            .arg("-Yes")
            .env("PATH", &path)
            .output()
            .unwrap()
    };
    let changes = || {
        std::fs::read_to_string(state.join("log"))
            .unwrap()
            .lines()
            .filter(|l| l.contains(" add ") || l.contains(" install "))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let first = run();
    let out = String::from_utf8_lossy(&first.stdout).replace("\r\n", "\n");
    assert!(first.status.success(), "{out}");
    assert_eq!(
        changes(),
        vec![
            "component add --toolchain stable clippy rust-src",
            "target add --toolchain stable wasm32-unknown-unknown",
            "toolchain install 1.80.0",
            "component add --toolchain 1.80.0 clippy",
            "toolchain install nightly",
        ],
        "{out}"
    );
    assert!(
        out.contains("Summary\nOK:\n  - rustup install rust stable (components: clippy, rust-src; targets: wasm32-unknown-unknown)\n  - rustup install rust 1.80.0 (components: clippy)\n  - rustup install rust nightly\nAlready done:\nNot done:\n"),
        "{out}"
    );
    let second = run();
    let out = String::from_utf8_lossy(&second.stdout).replace("\r\n", "\n");
    assert!(second.status.success(), "{out}");
    assert!(
        out.contains("Summary\nOK:\nAlready done:\n  - rustup install rust stable"),
        "{out}"
    );
    assert_eq!(changes().len(), 5);
}

#[test]
fn presence_checks_compare_whole_names() {
    let mut plan = windows_plan();
    let runtime = |tool, runtime: &str| Step::InstallRuntime {
        tool,
        runtime: runtime.into(),
        version: "20.1.1".into(),
        set_default: false,
        components: vec![],
        targets: vec![],
    };
    let global = |manager| Step::InstallGlobal {
        manager,
        name: "eslint".into(),
    };
    plan.steps = vec![
        Step::AddTap {
            manager: PackageManager::Scoop,
            name: "java".into(),
            url: None,
        },
        runtime(RuntimeTool::NvmWindows, "node"),
        runtime(RuntimeTool::Fnm, "node"),
        runtime(RuntimeTool::Volta, "node"),
        runtime(RuntimeTool::PyenvWin, "python"),
        global(GlobalManager::Npm),
        global(GlobalManager::Pnpm),
        global(GlobalManager::Pipx),
        global(GlobalManager::Cargo),
        Step::InstallExtension {
            editor: Editor::VsCode,
            id: "ms-python.python".into(),
        },
    ];
    let s = render_powershell(&plan);
    for check in [
        // `scoop bucket list` returns objects; compare the Name column, not the printed table.
        "  $installed = @((& scoop bucket list 2>$null).Name) -contains 'java'\n",
        "  $installed = @(& nvm list 2>$null | ForEach-Object { -split $_ }) -contains '20.1.1'\n",
        "  $installed = @(& fnm list 2>$null | ForEach-Object { -split $_ }) -contains 'v20.1.1'\n",
        "  $installed = @(& volta list 'node' --format plain 2>$null | ForEach-Object { -split $_ }) -contains 'node@20.1.1'\n",
        "  $installed = @(& pyenv versions --bare 2>$null | ForEach-Object { -split $_ }) -contains '20.1.1'\n",
        "  & npm ls -g --depth=0 'eslint' *> $null\n",
        "  $installed = @(& pnpm ls -g --depth=0 2>$null | ForEach-Object { ($_ -split ' ')[0] }) -contains 'eslint'\n",
        "  $installed = @(& pipx list --short 2>$null | ForEach-Object { ($_ -split ' ')[0] }) -contains 'eslint'\n",
        "  $installed = @(& cargo install --list 2>$null | ForEach-Object { ($_ -split ' ')[0] }) -contains 'eslint'\n",
        "  $installed = @(& code --list-extensions 2>$null) -contains 'ms-python.python'\n",
    ] {
        assert!(s.contains(check), "missing {check:?} in\n{s}");
    }
    assert!(!s.contains("Select-String"), "{s}");
}

#[test]
fn ssh_config_is_copied_as_usual_and_keys_are_listed_before_notes() {
    let s = render_powershell(&windows_plan());
    assert!(s.contains("$rc = Copy-RepotFile 'home/.ssh/config' (Join-Path $HOME '.ssh\\config')\nif ($rc -eq 0) { Add-Ok 'copy %USERPROFILE%\\.ssh\\config' }"));
    let tail = s.split("Write-Host 'Failed:'").nth(1).unwrap();
    assert!(tail.contains(
        "Write-Host 'Copy these SSH keys yourself (Repot does not export keys):'\n\
         Write-Host '  - ~/.ssh/id_ed25519 (files/home/.ssh/config line 7)'\n\
         Write-Host '  Copy each key and its .pub file from your old machine into %USERPROFILE%\\.ssh.'\n\
         Write-Host 'Notes:'\n\
         Write-Host 'files/home/.ssh/config line 6: UseKeychain was commented out because it is OS-specific.'\n"
    ), "{tail}");
    let mut plan = windows_plan();
    plan.notes.clear();
    let s = render_powershell(&plan);
    assert!(s.contains(
        "Write-Host '  - the keys you use (default names: id_ed25519, id_ecdsa, id_rsa)'"
    ));
    assert!(!s.contains("Write-Host 'Notes:'"));
    plan.steps
        .retain(|s| !matches!(s, Step::CopyFile { dest, .. } if dest.is_private()));
    assert!(!render_powershell(&plan).contains("SSH keys"));
}
