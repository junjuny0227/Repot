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
    }
}
fn sample_steps() -> Vec<Step> {
    vec![
        Step::AddTap {
            manager: PackageManager::Brew,
            name: "oven-sh/bun".into(),
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
        serde_json::json!({"buckets":[{"Name":"extras"}],"apps":[{"Name":"vlc","Source":"extras"},{"Name":"git","Source":"main"}]})
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
