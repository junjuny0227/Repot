use repot_core::{
    export::{
        step::{MappedFrom, Step, steps_for_target},
        *,
    },
    os::{LinuxFamily, Os},
    *,
};
fn package(id: &str, manager: PackageManager, kind: PackageKind, name: &str) -> Item {
    Item {
        id: id.into(),
        provider: "fixture".into(),
        category: Category::PackageManager,
        name: name.into(),
        version: None,
        description: None,
        catalog_id: None,
        origin: Origin::Command("fixture".into()),
        portability: Portability::Package,
        details: ItemDetails::Package { manager, kind },
        findings: vec![],
    }
}
fn items() -> Vec<Item> {
    use PackageKind::*;
    use PackageManager::*;
    vec![
        package("rg", Brew, Formula, "ripgrep"),
        package("git", Brew, Formula, "git"),
        package("firefox", Brew, Cask, "firefox"),
        package("rectangle", Brew, Cask, "rectangle"),
        package("missing", Brew, Formula, "missing"),
        package("tap", Brew, Tap, "owner/tap"),
    ]
}
fn bundle(items: &[Item], target: Target, format: Format) -> ExportBundle {
    build_export(
        &FakeEnv::new(Os::MacOs, "/Users/demo"),
        &ExportRequest {
            items,
            selection: &Selection {
                items: items.iter().map(|i| i.id.clone()).collect(),
                reveal_secret_lines: vec![],
            },
            source: Os::MacOs,
            target,
            format,
            meta: ExportMeta {
                repot_version: "0.2.0".into(),
                date: "2026-11-03".into(),
                generated_at: "2026-11-03 00:00 UTC".into(),
            },
        },
    )
}
#[test]
fn mapped_step_preserves_origin_and_adds_bucket_first() {
    let firefox = &items()[2];
    assert_eq!(
        steps_for_target(
            firefox,
            Os::MacOs,
            Target {
                os: Os::Windows,
                packages: Some(PackageManager::Scoop)
            }
        ),
        vec![
            Step::AddTap {
                manager: PackageManager::Scoop,
                name: "extras".into()
            },
            Step::InstallPackage {
                manager: PackageManager::Scoop,
                kind: PackageKind::Package,
                name: "extras/firefox".into(),
                mapped_from: Some(MappedFrom {
                    manager: PackageManager::Brew,
                    kind: PackageKind::Cask,
                    name: "firefox".into()
                })
            },
        ]
    );
    assert!(steps_for_target(&items()[3], Os::MacOs, Target::default_for(Os::Windows)).is_empty());
}
#[test]
fn mapped_targets_keep_first_case_insensitively_but_same_os_is_unchanged() {
    let mut items = vec![
        package(
            "first",
            PackageManager::Brew,
            PackageKind::Formula,
            "ripgrep",
        ),
        package(
            "second",
            PackageManager::Apt,
            PackageKind::Package,
            "RIPGREP",
        ),
    ];
    let b = bundle(&items, Target::default_for(Os::Windows), Format::Manifest);
    assert_eq!(
        b.warnings,
        vec![ExportWarning::DuplicateTarget {
            item_id: "second".into()
        }]
    );
    let winget = &b
        .files
        .iter()
        .find(|f| f.path == "winget.json")
        .unwrap()
        .contents;
    assert_eq!(winget.matches("BurntSushi.ripgrep.MSVC").count(), 1);
    items[1].details = items[0].details.clone();
    let b = bundle(&items, Target::default_for(Os::MacOs), Format::Manifest);
    assert!(b.warnings.is_empty());
    assert!(
        !b.files
            .iter()
            .find(|f| f.path == "README.md")
            .unwrap()
            .contents
            .contains("Mapped packages")
    );
}
#[test]
fn mapped_buckets_are_unique_and_safe_values_are_still_checked() {
    let items = vec![
        package("one", PackageManager::Brew, PackageKind::Cask, "firefox"),
        package(
            "two",
            PackageManager::Winget,
            PackageKind::Package,
            "Mozilla.Firefox",
        ),
        package(
            "unsafe",
            PackageManager::Brew,
            PackageKind::Formula,
            "bad name",
        ),
    ];
    let b = bundle(
        &items,
        Target {
            os: Os::Windows,
            packages: Some(PackageManager::Scoop),
        },
        Format::Script,
    );
    let script = &b.files[0].contents;
    assert_eq!(script.matches("& scoop bucket add 'extras'").count(), 1);
    assert!(
        script.find("& scoop bucket add 'extras'").unwrap()
            < script.find("& scoop install 'extras/firefox'").unwrap()
    );
    assert!(!script.contains("rectangle"));
    assert!(!script.contains("bad name"));
}
#[test]
fn cross_os_scripts_and_manifests() {
    let items = items();
    for (label, target) in [
        ("winget", Target::default_for(Os::Windows)),
        (
            "scoop",
            Target {
                os: Os::Windows,
                packages: Some(PackageManager::Scoop),
            },
        ),
        (
            "choco",
            Target {
                os: Os::Windows,
                packages: Some(PackageManager::Choco),
            },
        ),
        (
            "ubuntu",
            Target::default_for(Os::Linux(LinuxFamily::Debian)),
        ),
        (
            "fedora",
            Target::default_for(Os::Linux(LinuxFamily::Fedora)),
        ),
        ("arch", Target::default_for(Os::Linux(LinuxFamily::Arch))),
    ] {
        for format in [Format::Script, Format::Manifest] {
            let b = bundle(&items, target, format);
            for f in &b.files {
                assert!(!f.contents.contains("rectangle"));
                assert!(!f.contents.contains("install 'missing'"));
                assert!(!f.contents.contains("owner/tap"));
            }
            let readme = &b
                .files
                .iter()
                .find(|f| f.path == "README.md")
                .unwrap()
                .contents;
            assert!(readme.contains("## Mapped packages"));
            assert!(readme.contains("brew: ripgrep"));
            let output = b
                .files
                .iter()
                .map(|f| format!("=== {} ===\n{}", f.path, f.contents))
                .collect::<Vec<_>>()
                .join("\n");
            insta::assert_snapshot!(format!("{label}_{format:?}"), output);
        }
    }
}
