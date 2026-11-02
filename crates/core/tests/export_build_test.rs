use repot_core::{
    export::{meta::civil_from_days, *},
    os::{Os, Shell},
    *,
};

fn item(id: &str, name: &str, portability: Portability, details: ItemDetails) -> Item {
    Item {
        id: id.into(),
        name: name.into(),
        provider: "shell".into(),
        category: Category::Shell,
        version: None,
        description: None,
        origin: Origin::Command("fixture".into()),
        portability,
        details,
        findings: vec![],
    }
}
fn config(id: &str, source: &str, target: TargetPath, portability: Portability) -> Item {
    item(
        id,
        source,
        portability,
        ItemDetails::ConfigFile {
            source: source.into(),
            target,
        },
    )
}
fn fixture_machine() -> (FakeEnv, Vec<Item>) {
    let env = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_file("/Users/t/.zshrc", "export GITHUB_TOKEN=synthetic-secret\n")
        .with_file("/Users/t/.config/starship.toml", "add_newline = false\n");
    let mut runtime = item(
        "sdkman:java:21.0.2-tem",
        "java",
        Portability::Mapped(RuntimeTool::Sdkman),
        ItemDetails::Runtime {
            tool: RuntimeTool::Sdkman,
            runtime: "java".into(),
            is_default: false,
        },
    );
    runtime.version = Some("21.0.2-tem".into());
    (
        env,
        vec![
            item(
                "brew:formula:jq",
                "jq",
                Portability::SameOs,
                ItemDetails::Package {
                    manager: PackageManager::Brew,
                    kind: PackageKind::Formula,
                },
            ),
            config(
                "shell:file:.zshrc",
                "/Users/t/.zshrc",
                TargetPath::Home(".zshrc".into()),
                Portability::SameShell(Shell::Zsh),
            ),
            runtime,
            item(
                "npm:bad name",
                "bad name",
                Portability::Any,
                ItemDetails::GlobalPackage {
                    manager: GlobalManager::Npm,
                },
            ),
            config(
                "shell:file:.config/starship.toml",
                "/Users/t/.config/starship.toml",
                TargetPath::Home(".config/starship.toml".into()),
                Portability::Any,
            ),
        ],
    )
}
fn select_all(items: &[Item]) -> Selection {
    Selection {
        items: items.iter().map(|i| i.id.clone()).collect(),
        reveal_secret_lines: vec![],
    }
}
fn request<'a>(
    items: &'a [Item],
    selection: &'a Selection,
    source: Os,
    target: Os,
    format: Format,
) -> ExportRequest<'a> {
    ExportRequest {
        items,
        selection,
        source,
        target,
        format,
        meta: ExportMeta {
            repot_version: "0.1.0".into(),
            date: "2026-11-02".into(),
            generated_at: "2026-11-02 10:00 UTC".into(),
        },
    }
}
#[test]
fn civil_dates() {
    assert_eq!(civil_from_days(0), (1970, 1, 1));
    assert_eq!(civil_from_days(19782), (2024, 2, 29));
    assert_eq!(civil_from_days(20759), (2026, 11, 2));
}
fn warning_id(w: &ExportWarning) -> &str {
    match w {
        ExportWarning::Unavailable { item_id, .. }
        | ExportWarning::InvalidValue { item_id }
        | ExportWarning::DuplicateTarget { item_id }
        | ExportWarning::ReadFailed { item_id } => item_id,
    }
}
#[test]
fn filters_unavailable_and_invalid_items() {
    let (env, items) = fixture_machine();
    let sel = select_all(&items);
    let b = build_export(
        &env,
        &request(&items, &sel, Os::MacOs, Os::Windows, Format::Script),
    );
    assert_eq!(
        b.warnings.iter().map(warning_id).collect::<Vec<_>>(),
        vec![
            "brew:formula:jq",
            "shell:file:.zshrc",
            "sdkman:java:21.0.2-tem",
            "npm:bad name"
        ]
    );
    assert_eq!(
        b.files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>(),
        vec!["restore.ps1", "files/home/.config/starship.toml"]
    );
}
#[test]
fn script_for_unix_target_is_executable_and_masks_by_default() {
    let (env, items) = fixture_machine();
    let sel = select_all(&items);
    let b = build_export(
        &env,
        &request(&items, &sel, Os::MacOs, Os::MacOs, Format::Script),
    );
    let sh = &b.files[0];
    assert_eq!(sh.path, "restore.sh");
    assert!(sh.executable);
    let file = b
        .files
        .iter()
        .find(|f| f.path == "files/home/.zshrc")
        .unwrap();
    assert!(file.contents.contains("<REDACTED>"));
    assert!(
        sh.contents
            .contains("files/home/.zshrc line 1 (GITHUB_TOKEN)")
    );
}
#[test]
fn reveal_line_exports_original() {
    let (env, items) = fixture_machine();
    let mut sel = select_all(&items);
    sel.reveal_secret_lines
        .push(("shell:file:.zshrc".into(), 1));
    let b = build_export(
        &env,
        &request(&items, &sel, Os::MacOs, Os::MacOs, Format::Manifest),
    );
    assert_eq!(
        b.files
            .iter()
            .find(|f| f.path == "files/home/.zshrc")
            .unwrap()
            .contents,
        "export GITHUB_TOKEN=synthetic-secret\n"
    );
    assert!(b.files.iter().any(|f| f.path == "Brewfile"));
}
#[test]
fn duplicate_target_keeps_first() {
    let env = FakeEnv::new(Os::Windows, "C:/Users/t")
        .with_file("first.ps1", "first")
        .with_file("second.ps1", "second");
    let items = vec![
        config(
            "first",
            "first.ps1",
            TargetPath::PowerShellProfile,
            Portability::Any,
        ),
        config(
            "second",
            "second.ps1",
            TargetPath::PowerShellProfile,
            Portability::Any,
        ),
    ];
    let sel = select_all(&items);
    let b = build_export(
        &env,
        &request(&items, &sel, Os::Windows, Os::Windows, Format::Script),
    );
    let payloads: Vec<_> = b
        .files
        .iter()
        .filter(|f| f.path.starts_with("files/powershell/"))
        .collect();
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0].contents, "first");
    assert_eq!(
        b.warnings,
        vec![ExportWarning::DuplicateTarget {
            item_id: "second".into()
        }]
    );
}
#[test]
fn read_failure_becomes_warning() {
    let (_, items) = fixture_machine();
    let sel = select_all(&items);
    let env = FakeEnv::new(Os::MacOs, "/Users/t");
    let b = build_export(
        &env,
        &request(&items, &sel, Os::MacOs, Os::MacOs, Format::Script),
    );
    assert!(b.warnings.iter().any(
        |w| matches!(w, ExportWarning::ReadFailed { item_id } if item_id == "shell:file:.zshrc")
    ));
    assert!(b.files.iter().any(|f| f.path == "restore.sh"));
}

#[test]
fn runtime_display_name_is_not_validated_but_package_name_is() {
    let mut runtime = item(
        "rustup:rust:stable",
        "Rust toolchain",
        Portability::Mapped(RuntimeTool::Rustup),
        ItemDetails::Runtime {
            tool: RuntimeTool::Rustup,
            runtime: "rust".into(),
            is_default: true,
        },
    );
    runtime.version = Some("stable".into());
    let package = item(
        "npm:unsafe",
        "unsafe name",
        Portability::Any,
        ItemDetails::GlobalPackage {
            manager: GlobalManager::Npm,
        },
    );
    let items = vec![runtime, package];
    let selection = select_all(&items);
    let env = FakeEnv::new(Os::MacOs, "/Users/t");
    let bundle = build_export(
        &env,
        &request(&items, &selection, Os::MacOs, Os::MacOs, Format::Script),
    );

    assert_eq!(
        bundle.warnings,
        vec![ExportWarning::InvalidValue {
            item_id: "npm:unsafe".into(),
        }]
    );
    let script = &bundle
        .files
        .iter()
        .find(|file| file.path == "restore.sh")
        .unwrap()
        .contents;
    assert!(script.contains("rustup toolchain install 'stable'"));
}

#[test]
fn package_version_is_not_validated_because_it_is_not_exported() {
    // brew cask versions contain commas (e.g. "17.0.21.0.3,17.70.113"), but packages are
    // installed at the latest version, so the scanned version never reaches the script.
    let mut cask = item(
        "brew:cask:zulu@17",
        "zulu@17",
        Portability::SameOs,
        ItemDetails::Package {
            manager: PackageManager::Brew,
            kind: PackageKind::Cask,
        },
    );
    cask.version = Some("17.0.21.0.3,17.70.113".into());
    let mut runtime = item(
        "nvm:node:bad",
        "Node.js",
        Portability::Mapped(RuntimeTool::Nvm),
        ItemDetails::Runtime {
            tool: RuntimeTool::Nvm,
            runtime: "node".into(),
            is_default: false,
        },
    );
    runtime.version = Some("20,1".into());
    let items = vec![cask, runtime];
    let sel = select_all(&items);
    let env = FakeEnv::new(Os::MacOs, "/Users/t");
    let b = build_export(
        &env,
        &request(&items, &sel, Os::MacOs, Os::MacOs, Format::Script),
    );
    assert_eq!(
        b.warnings.iter().map(warning_id).collect::<Vec<_>>(),
        vec!["nvm:node:bad"]
    );
    let script = &b
        .files
        .iter()
        .find(|f| f.path == "restore.sh")
        .unwrap()
        .contents;
    assert!(script.contains("zulu@17"));
}
