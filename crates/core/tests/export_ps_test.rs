use repot_core::{
    export::{
        payload::MaskedLine,
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
        ],
        masked: vec![MaskedLine {
            file: "home/.gitconfig".into(),
            line: 9,
            name: None,
        }],
        notes: vec![],
    }
}
#[test]
fn native_output_is_assigned_before_conditionals() {
    let s = render_powershell(&windows_plan());
    assert!(s.contains("function Test-Tool($Name)"));
    assert!(!s.contains("Test-Have"));
    assert!(!s.contains("if (& "));
    assert!(s.contains("$installed = & scoop bucket list 2>$null | Select-String"));
    assert!(s.contains("if ($installed) { Add-Done"));
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
