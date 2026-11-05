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
                url: None,
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
    assert!(String::from_utf8_lossy(&second.stdout).contains("already done: copy ~/.zshrc"));
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
    // A missing tool is not a failure, but the summary says what was not done and why.
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("brew not found. Install it first: https://brew.sh"));
    // The preflight names the missing tool before anything runs.
    let preflight = stdout
        .find("Tools this script uses:\n  brew: not found yet. Install it from https://brew.sh\n");
    let first_step = stdout.find("== Packages ==");
    assert!(
        matches!((preflight, first_step), (Some(p), Some(s)) if p < s),
        "{stdout}"
    );
    let summary = stdout.split("Summary\n").nth(1).unwrap();
    assert_eq!(
        summary,
        "OK:\nAlready done:\nNot done:\n  - brew install jq (brew not found)\n\
         To finish these, install the missing tools and run this script again:\n  brew: https://brew.sh\n\
         Failed:\n",
        "{stdout}"
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

#[cfg(unix)]
#[test]
fn jetbrains_files_go_to_the_newest_version_folder_and_plugins_install_once() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let target = current_target();
    let base = if target == Os::MacOs {
        home.join("Library/Application Support/JetBrains")
    } else {
        home.join(".config/JetBrains")
    };
    let plugins_of = |folder: &str| {
        if target == Os::MacOs {
            base.join(folder).join("plugins")
        } else {
            home.join(".local/share/JetBrains").join(folder)
        }
    };
    std::fs::create_dir_all(base.join("IntelliJIdea2025.3")).unwrap();
    std::fs::create_dir_all(base.join("IntelliJIdea2026.10")).unwrap();
    std::fs::create_dir_all(base.join("IntelliJIdea2026.2")).unwrap();
    std::fs::create_dir_all(plugins_of("IntelliJIdea2026.10").join("present-plugin")).unwrap();
    std::fs::create_dir_all(dir.path().join("files/jetbrains/IntelliJIdea/codestyles")).unwrap();
    std::fs::write(
        dir.path()
            .join("files/jetbrains/IntelliJIdea/codestyles/Default.xml"),
        "<code_scheme/>\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("files/jetbrains/WebStorm/keymaps")).unwrap();
    std::fs::write(
        dir.path().join("files/jetbrains/WebStorm/keymaps/K.xml"),
        "<keymap/>\n",
    )
    .unwrap();
    // A stand-in launcher on PATH that records what it was asked to do.
    let bin = dir.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let log = dir.path().join("idea.log");
    std::fs::write(
        bin.join("idea"),
        format!("#!/bin/sh\necho \"$@\" >> '{}'\n", log.display()),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(bin.join("idea"), std::fs::Permissions::from_mode(0o755)).unwrap();
    let p = plan(
        target,
        vec![
            Step::InstallIdePlugin {
                ide: JetBrainsIde::IntelliJIdea,
                id: "present.plugin".into(),
                folder: "present-plugin".into(),
            },
            Step::InstallIdePlugin {
                ide: JetBrainsIde::IntelliJIdea,
                id: "com.github.copilot".into(),
                folder: "github-copilot-intellij".into(),
            },
            Step::CopyFile {
                src: "jetbrains/IntelliJIdea/codestyles/Default.xml".into(),
                dest: TargetPath::JetBrains {
                    ide: JetBrainsIde::IntelliJIdea,
                    version: "2024.1".into(),
                    rel: "codestyles/Default.xml".into(),
                },
            },
            // WebStorm never ran here: its folder gets the source machine's version.
            Step::CopyFile {
                src: "jetbrains/WebStorm/keymaps/K.xml".into(),
                dest: TargetPath::JetBrains {
                    ide: JetBrainsIde::WebStorm,
                    version: "2026.1".into(),
                    rel: "keymaps/K.xml".into(),
                },
            },
        ],
    );
    let path = format!("{}:/usr/bin:/bin", bin.display());
    // CI runners set XDG_*; point them inside the fake home like a default desktop.
    let config = home.join(".config");
    let data = home.join(".local/share");
    let out = run_script(
        &p,
        dir.path(),
        &[
            ("HOME", home.to_str().unwrap()),
            ("PATH", &path),
            ("XDG_CONFIG_HOME", config.to_str().unwrap()),
            ("XDG_DATA_HOME", data.to_str().unwrap()),
        ],
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{stdout}");
    assert!(stdout.contains("  IntelliJ IDEA: found"), "{stdout}");
    // 2026.10 is newer than 2026.2 (numeric, not text order).
    assert_eq!(
        std::fs::read_to_string(base.join("IntelliJIdea2026.10/codestyles/Default.xml")).unwrap(),
        "<code_scheme/>\n"
    );
    assert!(base.join("WebStorm2026.1/keymaps/K.xml").exists());
    assert!(
        stdout.contains("already done: idea installPlugins present.plugin"),
        "{stdout}"
    );
    assert_eq!(
        std::fs::read_to_string(&log).unwrap(),
        "installPlugins com.github.copilot\n"
    );
}

#[cfg(unix)]
#[test]
fn jetbrains_plugins_without_the_ide_are_not_done() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let p = plan(
        current_target(),
        vec![Step::InstallIdePlugin {
            ide: JetBrainsIde::WebStorm,
            id: "dev.blachut.svelte.lang".into(),
            folder: "svelte".into(),
        }],
    );
    let config = home.join(".config");
    let out = run_script(
        &p,
        dir.path(),
        &[
            ("HOME", home.to_str().unwrap()),
            ("PATH", "/usr/bin:/bin"),
            ("XDG_CONFIG_HOME", config.to_str().unwrap()),
        ],
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{stdout}");
    assert!(
        stdout.contains("  - webstorm installPlugins dev.blachut.svelte.lang (WebStorm not found)"),
        "{stdout}"
    );
    assert!(stdout.contains("  WebStorm: https://www.jetbrains.com/toolbox-app/"));
}
