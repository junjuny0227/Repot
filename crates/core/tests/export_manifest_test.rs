use repot_core::{
    export::{
        manifest::render_manifests,
        payload::{ExportNote, MaskedLine},
        script::ScriptHeader,
        step::Step,
    },
    model::*,
    os::{LinuxFamily, Os},
};
fn header(target: Os) -> ScriptHeader {
    ScriptHeader {
        repot_version: "0.1.0".into(),
        source: Os::MacOs,
        target,
        generated_at: "2026-11-02 10:00 UTC".into(),
    }
}
fn pkg(manager: PackageManager, kind: PackageKind, name: &str) -> Step {
    Step::InstallPackage {
        manager,
        kind,
        name: name.into(),
        mapped_from: None,
    }
}
fn rt(runtime: &str, version: &str, set_default: bool) -> Step {
    Step::InstallRuntime {
        tool: if runtime == "rust" {
            RuntimeTool::Rustup
        } else {
            RuntimeTool::Nvm
        },
        runtime: runtime.into(),
        version: version.into(),
        set_default,
        components: vec![],
        targets: vec![],
    }
}
fn sample_steps() -> Vec<Step> {
    vec![
        Step::AddTap {
            manager: PackageManager::Brew,
            name: "oven-sh/bun".into(),
            url: None,
        },
        pkg(PackageManager::Brew, PackageKind::Formula, "jq"),
        pkg(
            PackageManager::Brew,
            PackageKind::Cask,
            "visual-studio-code",
        ),
        rt("node", "18.20.4", false),
        rt("node", "20.11.0", true),
        rt("rust", "stable", true),
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
            src: "home/.ssh/config".into(),
            dest: TargetPath::Home(".ssh/config".into()),
        },
        Step::GitClone {
            url: "https://github.com/zsh-users/zsh-autosuggestions".into(),
            dest: TargetPath::Home(".oh-my-zsh/custom/plugins/zsh-autosuggestions".into()),
        },
    ]
}
fn sample_masked() -> Vec<MaskedLine> {
    vec![MaskedLine {
        file: "home/.zshrc".into(),
        line: 3,
        name: Some("GITHUB_TOKEN".into()),
    }]
}
fn sample_notes() -> Vec<ExportNote> {
    vec![
        ExportNote::CommentedOsLine {
            file: "home/.gitconfig".into(),
            line: 5,
            key: "credential.helper".into(),
        },
        ExportNote::KeybindingsMayDiffer {
            file: "vscode/keybindings.json".into(),
        },
        ExportNote::SshKey {
            file: "home/.ssh/config".into(),
            line: 7,
            path: "~/.ssh/id_ed25519".into(),
        },
        ExportNote::SshKey {
            file: "home/.ssh/config".into(),
            line: 12,
            path: "~/.ssh/id_ed25519".into(),
        },
    ]
}
#[test]
fn brewfile_and_tool_versions() {
    let files = render_manifests(&header(Os::MacOs), &sample_steps(), &[], &[]);
    let get = |p: &str| {
        files
            .iter()
            .find(|f| f.path == p)
            .map(|f| f.contents.clone())
    };
    assert_eq!(
        get("Brewfile").unwrap(),
        "tap \"oven-sh/bun\"\nbrew \"jq\"\ncask \"visual-studio-code\"\n"
    );
    assert_eq!(get(".tool-versions").unwrap(), "node 20.11.0 18.20.4\n");
    assert!(
        get("README.md")
            .unwrap()
            .contains("rustup toolchain install stable")
    );
    assert!(get("winget.json").is_none());
    assert!(files.iter().all(|f| !f.executable));
}
fn windows_steps() -> Vec<Step> {
    vec![
        pkg(PackageManager::Winget, PackageKind::Package, "Git.Git"),
        Step::AddTap {
            manager: PackageManager::Scoop,
            name: "extras".into(),
            url: None,
        },
        Step::AddTap {
            manager: PackageManager::Scoop,
            name: "repot".into(),
            url: Some("https://github.com/junjuny0227/scoop-bucket".into()),
        },
        pkg(PackageManager::Scoop, PackageKind::Package, "extras/vlc"),
        pkg(PackageManager::Scoop, PackageKind::Package, "git"),
        pkg(PackageManager::Choco, PackageKind::Package, "7zip"),
    ]
}
#[test]
fn windows_manifests() {
    let files = render_manifests(&header(Os::Windows), &windows_steps(), &[], &[]);
    let get = |p: &str| files.iter().find(|f| f.path == p).unwrap().contents.clone();
    let winget: serde_json::Value = serde_json::from_str(&get("winget.json")).unwrap();
    assert_eq!(
        winget["Sources"][0]["Packages"][0]["PackageIdentifier"],
        "Git.Git"
    );
    let scoop: serde_json::Value = serde_json::from_str(&get("scoop.json")).unwrap();
    assert_eq!(
        scoop,
        serde_json::json!({"buckets":[{"Name":"extras"},{"Name":"repot","Source":"https://github.com/junjuny0227/scoop-bucket"}],"apps":[{"Name":"vlc","Source":"extras"},{"Name":"git","Source":"main"}]})
    );
    assert!(get("choco-packages.config").contains(r#"<package id="7zip" />"#));
    assert!(get("README.md").contains("winget import -i winget.json"));
    insta::assert_snapshot!(
        files
            .iter()
            .map(|f| format!("--- {} ---\n{}", f.path, f.contents))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
#[test]
fn readme_snapshot() {
    insta::assert_snapshot!(
        render_manifests(
            &header(Os::MacOs),
            &sample_steps(),
            &sample_masked(),
            &sample_notes()
        )
        .into_iter()
        .find(|f| f.path == "README.md")
        .unwrap()
        .contents
    );
}
#[test]
fn native_manifests_snapshot() {
    let mut steps = sample_steps();
    for m in [
        PackageManager::Apt,
        PackageManager::Dnf,
        PackageManager::Pacman,
    ] {
        steps.push(pkg(m, PackageKind::Package, "ripgrep"));
    }
    for m in [
        GlobalManager::Pnpm,
        GlobalManager::Pipx,
        GlobalManager::Cargo,
    ] {
        steps.push(Step::InstallGlobal {
            manager: m,
            name: "tool".into(),
        });
    }
    steps.push(Step::InstallExtension {
        editor: Editor::Cursor,
        id: "example.extension".into(),
    });
    steps.push(rt("python", "3.12.0", false));
    let files = render_manifests(&header(Os::Linux(LinuxFamily::Debian)), &steps, &[], &[]);
    insta::assert_snapshot!(
        files
            .iter()
            .map(|f| format!("--- {} ---\n{}", f.path, f.contents))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
#[test]
fn empty_export_has_only_readme() {
    let files = render_manifests(&header(Os::MacOs), &[], &[], &[]);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "README.md");
    assert!(!files[0].contents.contains("## Masked secrets"));
    assert!(!files[0].contents.contains("## Notes"));
}

#[test]
fn readme_lists_rustup_components_and_targets() {
    let steps = vec![Step::InstallRuntime {
        tool: RuntimeTool::Rustup,
        runtime: "rust".into(),
        version: "stable".into(),
        set_default: true,
        components: vec!["clippy".into(), "rust-src".into()],
        targets: vec!["wasm32-unknown-unknown".into()],
    }];
    let files = render_manifests(&header(Os::MacOs), &steps, &[], &[]);
    let readme = &files
        .iter()
        .find(|f| f.path == "README.md")
        .unwrap()
        .contents;
    assert!(
        readme.contains(
            "rustup toolchain install stable\nrustup component add --toolchain stable clippy rust-src\nrustup target add --toolchain stable wasm32-unknown-unknown\n"
        ),
        "{readme}"
    );
}

#[test]
fn readme_lists_ssh_keys_and_permissions_only_where_they_apply() {
    let readme = |target: Os, steps: &[Step], notes: &[ExportNote]| {
        render_manifests(&header(target), steps, &[], notes)
            .into_iter()
            .find(|f| f.path == "README.md")
            .unwrap()
            .contents
    };
    let mac = readme(Os::MacOs, &sample_steps(), &sample_notes());
    assert_eq!(mac.matches("~/.ssh/id_ed25519").count(), 1, "{mac}");
    assert!(mac.contains("| `files/home/.ssh/config` | `~/.ssh/config` |\n\nOpenSSH rejects SSH files that others can write: if you create `~/.ssh`, run `chmod 700 ~/.ssh`, and run `chmod 600` on each file you copy into it.\n"));
    // Keys are not general notes.
    let notes = mac.split("## Notes").nth(1).unwrap();
    assert!(!notes.contains("id_ed25519"));
    let windows = readme(Os::Windows, &sample_steps(), &sample_notes());
    assert!(!windows.contains("chmod"));
    assert!(windows.contains("\n## SSH keys\n\nCopy these SSH keys yourself (Repot does not export keys):\n  - ~/.ssh/id_ed25519 (files/home/.ssh/config line 7)\n  Copy each key and its .pub file from your old machine into %USERPROFILE%\\.ssh.\n"));
    let without_ssh: Vec<Step> = sample_steps()
        .into_iter()
        .filter(|s| !matches!(s, Step::CopyFile { dest, .. } if dest.is_private()))
        .collect();
    let plain = readme(Os::MacOs, &without_ssh, &[]);
    assert!(!plain.contains("SSH") && !plain.contains("chmod"));
}
