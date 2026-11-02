use repot_core::{export::step::*, os::*, *};

fn item(details: ItemDetails, name: &str, version: Option<&str>) -> Item {
    Item {
        id: name.into(),
        provider: "test".into(),
        category: Category::PackageManager,
        name: name.into(),
        version: version.map(str::to_owned),
        description: None,
        origin: Origin::Command("test".into()),
        portability: Portability::Any,
        details,
        findings: vec![],
    }
}
#[test]
fn maps_items_to_steps() {
    use PackageKind::*;
    use PackageManager::*;
    let pkg = |manager, kind, name| item(ItemDetails::Package { manager, kind }, name, None);
    assert_eq!(
        steps_for(&pkg(Brew, Tap, "oven-sh/bun"), Os::MacOs),
        vec![Step::AddTap {
            manager: Brew,
            name: "oven-sh/bun".into()
        }]
    );
    assert_eq!(
        steps_for(&pkg(Brew, Cask, "code"), Os::MacOs),
        vec![Step::InstallPackage {
            manager: Brew,
            kind: Cask,
            name: "code".into()
        }]
    );
    let rt = |tool| {
        item(
            ItemDetails::Runtime {
                tool,
                runtime: "node".into(),
                is_default: true,
            },
            "node",
            Some("20.11.0"),
        )
    };
    assert_eq!(
        steps_for(&rt(RuntimeTool::Nvm), Os::Windows),
        vec![Step::InstallRuntime {
            tool: RuntimeTool::NvmWindows,
            runtime: "node".into(),
            version: "20.11.0".into(),
            set_default: true
        }]
    );
    assert!(steps_for(&rt(RuntimeTool::Sdkman), Os::Windows).is_empty());
    assert_eq!(
        steps_for(
            &item(
                ItemDetails::Extension {
                    editor: Editor::Cursor
                },
                "prettier",
                None
            ),
            Os::Linux(LinuxFamily::Arch)
        ),
        vec![Step::InstallExtension {
            editor: Editor::Cursor,
            id: "prettier".into()
        }]
    );
    assert_eq!(
        steps_for(
            &item(
                ItemDetails::GlobalPackage {
                    manager: GlobalManager::Npm
                },
                "typescript",
                None
            ),
            Os::MacOs
        ),
        vec![Step::InstallGlobal {
            manager: GlobalManager::Npm,
            name: "typescript".into()
        }]
    );
    let dest = TargetPath::Home(".oh-my-zsh/custom/plugins/test".into());
    assert_eq!(
        steps_for(
            &item(
                ItemDetails::ShellPlugin {
                    url: "https://github.com/a/b".into(),
                    target: dest.clone()
                },
                "test",
                None
            ),
            Os::MacOs
        ),
        vec![Step::GitClone {
            url: "https://github.com/a/b".into(),
            dest
        }]
    );
}
#[test]
fn sorts_by_section_with_taps_first_stably() {
    let pkg = |name: &str| Step::InstallPackage {
        manager: PackageManager::Brew,
        kind: PackageKind::Formula,
        name: name.into(),
    };
    let copy = Step::CopyFile {
        src: "home/.zshrc".into(),
        dest: TargetPath::Home(".zshrc".into()),
    };
    let global = Step::InstallGlobal {
        manager: GlobalManager::Npm,
        name: "typescript".into(),
    };
    let tap = Step::AddTap {
        manager: PackageManager::Brew,
        name: "oven-sh/bun".into(),
    };
    let mut steps = vec![
        copy.clone(),
        global.clone(),
        pkg("jq"),
        tap.clone(),
        pkg("rg"),
    ];
    sort_steps(&mut steps);
    assert_eq!(steps, vec![tap, pkg("jq"), pkg("rg"), global, copy]);
}
#[test]
fn safe_values() {
    for ok in [
        "ripgrep",
        "@anthropic-ai/claude-code",
        "Git.Git",
        "extras/vlc",
        "20.11.0",
        "https://github.com/a/b",
        "git@github.com:a/b.git",
    ] {
        assert!(is_safe_value(ok), "{ok}");
    }
    for bad in [
        "",
        "a b",
        "x;rm -rf ~",
        "$(evil)",
        "a'b",
        "a\"b",
        "a`b",
        "한글",
        "a\nb",
    ] {
        assert!(!is_safe_value(bad), "{bad}");
    }
}
#[test]
fn same_kind_and_selection_serde() {
    assert!(Os::Linux(LinuxFamily::Debian).same_kind(Os::Linux(LinuxFamily::Arch)));
    assert!(Os::MacOs.same_kind(Os::MacOs));
    assert!(Os::Windows.same_kind(Os::Windows));
    assert!(!Os::MacOs.same_kind(Os::Windows));
    let selection = Selection {
        items: vec!["test".into()],
        reveal_secret_lines: vec![("test".into(), 9)],
    };
    let json = serde_json::to_value(&selection).unwrap();
    assert_eq!(
        json,
        serde_json::json!({"items":["test"],"revealSecretLines":[["test",9]]})
    );
    assert_eq!(
        serde_json::from_value::<Selection>(json).unwrap(),
        selection
    );
}
#[test]
fn guide_urls() {
    assert_eq!(
        guide_url("nvm-windows"),
        "https://github.com/coreybutler/nvm-windows"
    );
    assert_eq!(guide_url("npm"), guide_url("pnpm"));
    for key in ["apt", "dnf", "pacman", "unknown"] {
        assert_eq!(guide_url(key), "");
    }
}
