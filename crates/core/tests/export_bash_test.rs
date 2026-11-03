#[cfg(unix)]
use repot_core::os::LinuxFamily;
use repot_core::{
    export::{
        payload::{ExportNote, MaskedLine},
        script::{ScriptHeader, ScriptPlan, bash::render_bash},
        step::Step,
    },
    model::*,
    os::Os,
};

fn header(target: Os) -> ScriptHeader {
    ScriptHeader {
        repot_version: "0.1.0".into(),
        source: Os::MacOs,
        target,
        generated_at: "2026-11-02 10:00 UTC".into(),
    }
}
fn plan(target: Os, steps: Vec<Step>) -> ScriptPlan {
    ScriptPlan {
        header: header(target),
        steps,
        masked: vec![],
        notes: vec![],
    }
}
fn sample_plan(target: Os) -> ScriptPlan {
    let mut p = plan(
        target,
        vec![
            Step::AddTap {
                manager: PackageManager::Brew,
                name: "oven-sh/bun".into(),
            },
            Step::InstallPackage {
                manager: PackageManager::Brew,
                kind: PackageKind::Formula,
                name: "jq".into(),
                mapped_from: None,
            },
            Step::InstallPackage {
                manager: PackageManager::Brew,
                kind: PackageKind::Cask,
                name: "visual-studio-code".into(),
                mapped_from: None,
            },
            Step::InstallRuntime {
                tool: RuntimeTool::Nvm,
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
                src: "home/.zshrc".into(),
                dest: TargetPath::Home(".zshrc".into()),
            },
            Step::CopyFile {
                src: "home/it's.json".into(),
                dest: TargetPath::Home("it's.json".into()),
            },
            Step::GitClone {
                url: "https://github.com/zsh-users/zsh-autosuggestions".into(),
                dest: TargetPath::Home(".oh-my-zsh/custom/plugins/zsh-autosuggestions".into()),
            },
        ],
    );
    p.masked.push(MaskedLine {
        file: "home/.zshrc".into(),
        line: 3,
        name: Some("GITHUB_TOKEN".into()),
    });
    p.notes.push(ExportNote::CommentedOsLine {
        file: "home/.gitconfig".into(),
        line: 5,
        key: "credential.helper".into(),
    });
    p
}
#[test]
fn bash_snapshot_macos() {
    insta::assert_snapshot!(render_bash(&sample_plan(Os::MacOs)));
}
// Unix only: on Windows runners `bash` may resolve to the WSL launcher, which cannot
// check a script passed through `-c`. The generated bash is linted and run on the macOS and
// Ubuntu runners (ci.yml scripts job), which is where it is executed for real.
#[cfg(unix)]
#[test]
fn bash_syntax_is_valid() {
    for os in [Os::MacOs, Os::Linux(LinuxFamily::Debian)] {
        let out = std::process::Command::new("bash")
            .args(["-n", "-c", &render_bash(&sample_plan(os))])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
#[cfg(unix)]
fn current_target() -> Os {
    if cfg!(target_os = "macos") {
        Os::MacOs
    } else {
        Os::Linux(LinuxFamily::Debian)
    }
}
#[cfg(unix)]
fn copy_only_plan() -> ScriptPlan {
    plan(
        current_target(),
        vec![Step::CopyFile {
            src: "home/.zshrc".into(),
            dest: TargetPath::Home(".zshrc".into()),
        }],
    )
}
#[cfg(unix)]
fn run_script(
    p: &ScriptPlan,
    dir: &std::path::Path,
    envs: &[(&str, &str)],
) -> std::process::Output {
    std::fs::write(dir.join("restore.sh"), render_bash(p)).unwrap();
    std::process::Command::new("bash")
        .arg(dir.join("restore.sh"))
        .arg("--yes")
        .envs(envs.iter().copied())
        .output()
        .unwrap()
}
#[cfg(unix)]
#[test]
fn copy_step_backs_up_once() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join(".zshrc"), "old\n").unwrap();
    std::fs::create_dir_all(dir.path().join("files/home")).unwrap();
    std::fs::write(dir.path().join("files/home/.zshrc"), "new\n").unwrap();
    let p = copy_only_plan();
    let envs = [("HOME", home.to_str().unwrap())];
    assert!(run_script(&p, dir.path(), &envs).status.success());
    assert_eq!(
        std::fs::read_to_string(home.join(".zshrc")).unwrap(),
        "new\n"
    );
    let backups = || {
        std::fs::read_dir(&home)
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".zshrc.repot-backup-")
            })
            .count()
    };
    assert_eq!(backups(), 1);
    let second = run_script(&p, dir.path(), &envs);
    assert!(second.status.success());
    assert!(String::from_utf8_lossy(&second.stdout).contains("skipped: copy ~/.zshrc"));
    assert_eq!(backups(), 1);
}
#[cfg(unix)]
#[test]
fn missing_tool_is_skipped_and_failure_sets_exit_code() {
    let dir = tempfile::tempdir().unwrap();
    let p = plan(
        current_target(),
        vec![Step::InstallPackage {
            manager: PackageManager::Brew,
            kind: PackageKind::Formula,
            name: "jq".into(),
            mapped_from: None,
        }],
    );
    let out = run_script(&p, dir.path(), &[("PATH", "/usr/bin:/bin")]);
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stdout)
            .contains("brew not found. Install it first: https://brew.sh")
    );
    let home = dir.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    assert_eq!(
        run_script(
            &copy_only_plan(),
            dir.path(),
            &[("HOME", home.to_str().unwrap())]
        )
        .status
        .code(),
        Some(1)
    );
}
