use repot_core::{
    compare::Presence,
    export::{ExportMeta, Format},
    os::Os,
    scan::ScanResult,
    session::{OpenError, Session},
    snapshot::SnapshotError,
    *,
};
use std::collections::HashMap;
use std::path::Path;

fn global(name: &str) -> Item {
    Item {
        id: format!("npm:{name}"),
        name: name.into(),
        provider: "npm".into(),
        category: Category::GlobalPackage,
        version: None,
        description: None,
        catalog_id: None,
        origin: Origin::Command("fixture".into()),
        portability: Portability::Any,
        details: ItemDetails::GlobalPackage {
            manager: GlobalManager::Npm,
        },
        findings: vec![],
    }
}
/// Usable only on the OS it came from.
fn mac_only(name: &str) -> Item {
    Item {
        id: format!("mac:{name}"),
        portability: Portability::SameOs,
        ..global(name)
    }
}
fn zshrc() -> Item {
    Item {
        id: "shell:file:.zshrc".into(),
        name: ".zshrc".into(),
        provider: "shell".into(),
        category: Category::Shell,
        details: ItemDetails::ConfigFile {
            source: "/Users/t/.zshrc".into(),
            target: TargetPath::Home(".zshrc".into()),
        },
        ..global("zshrc")
    }
}
fn scan_of(items: Vec<Item>) -> ScanResult {
    ScanResult {
        items,
        providers: vec![],
    }
}
fn meta() -> ExportMeta {
    ExportMeta {
        repot_version: "0.7.0".into(),
        date: "2026-11-04".into(),
        generated_at: "2026-11-04T00:00:00Z".into(),
    }
}
fn mac() -> Target {
    Target::default_for(Os::MacOs)
}
fn windows() -> Target {
    Target::default_for(Os::Windows)
}
fn env() -> FakeEnv {
    FakeEnv::new(Os::MacOs, "/Users/t").with_file("/Users/t/.zshrc", "export A=1\n")
}

#[test]
fn a_scan_starts_with_every_usable_item() {
    let session = Session::from_scan(
        Os::MacOs,
        scan_of(vec![global("typescript"), mac_only("rectangle"), zshrc()]),
    );
    assert!(session.not_exported().is_empty());
    assert_eq!(
        session.default_selection(mac(), None),
        ["npm:typescript", "mac:rectangle", "shell:file:.zshrc"]
    );
    assert_eq!(
        session.default_selection(windows(), None),
        ["npm:typescript", "shell:file:.zshrc"]
    );
}

/// Exports `items` for `target` and returns the written repot.json text.
fn repot_file(items: Vec<Item>, selected: &[&str], target: Target) -> String {
    let session = Session::from_scan(Os::MacOs, scan_of(items));
    let selection = Selection {
        items: selected.iter().map(|s| s.to_string()).collect(),
        reveal_secret_lines: vec![],
    };
    let bundle = session
        .preview(&env(), &selection, target, Format::Script, meta())
        .unwrap();
    bundle
        .files
        .into_iter()
        .find(|f| f.path == "repot.json")
        .unwrap()
        .contents
}

#[test]
fn an_opened_file_leaves_items_outside_its_export_unselected() {
    // Exported for Windows: the Mac-only item is stored but not exported (plan 13).
    let text = repot_file(
        vec![global("typescript"), mac_only("rectangle"), zshrc()],
        &["npm:typescript", "shell:file:.zshrc"],
        windows(),
    );
    let env = env().with_file("/x/repot.json", &text);
    let (session, info) = Session::open(&env, Path::new("/x/repot.json")).unwrap();
    assert_eq!(info.repot_version, "0.7.0");
    assert_eq!(info.skipped_items, 0);
    assert!(session.not_exported().contains("mac:rectangle"));
    // Usable again on a Mac, but still not selected at first.
    assert_eq!(
        session.default_selection(mac(), None),
        ["npm:typescript", "shell:file:.zshrc"]
    );
}

#[test]
fn after_comparing_only_missing_items_are_selected() {
    let text = repot_file(
        vec![global("typescript"), global("eslint"), zshrc()],
        &["npm:typescript", "npm:eslint", "shell:file:.zshrc"],
        mac(),
    );
    let env = env().with_file("/x/repot.json", &text);
    let (mut session, _) = Session::open(&env, Path::new("/x/repot.json")).unwrap();
    // No comparison yet: nothing to judge.
    assert!(session.presence(&env, Os::MacOs, mac()).is_empty());
    session.machine = Some(scan_of(vec![global("typescript"), zshrc()]));
    // Another OS than this machine's: no judgment either.
    assert!(session.presence(&env, Os::MacOs, windows()).is_empty());
    let presence = session.presence(&env, Os::MacOs, mac());
    assert_eq!(presence["npm:typescript"], Presence::Present);
    assert_eq!(presence["npm:eslint"], Presence::Missing);
    assert_eq!(presence["shell:file:.zshrc"], Presence::Present);
    assert_eq!(
        session.default_selection(mac(), Some(&presence)),
        ["npm:eslint"]
    );
    // An empty comparison selects nothing rather than everything.
    assert!(
        session
            .default_selection(mac(), Some(&HashMap::new()))
            .is_empty()
    );
}

#[test]
fn opening_refuses_missing_large_and_foreign_files() {
    let big = "x".repeat(repot_core::snapshot::MAX_BYTES + 1);
    let env = env()
        .with_file("/x/big.json", &big)
        .with_file("/x/other.json", "{\"name\":\"not repot\"}");
    let open = |p: &str| Session::open(&env, Path::new(p)).map(|_| ()).unwrap_err();
    assert_eq!(open("/x/none.json"), OpenError::ReadFailed);
    assert_eq!(
        open("/x/big.json"),
        OpenError::Snapshot(SnapshotError::TooLarge)
    );
    assert_eq!(
        open("/x/other.json"),
        OpenError::Snapshot(SnapshotError::NotRepotFile)
    );
}

#[test]
fn preview_reads_configs_from_the_machine_or_the_file() {
    let items = vec![global("typescript"), zshrc()];
    let all = ["npm:typescript", "shell:file:.zshrc"];
    let text = repot_file(items.clone(), &all, mac());
    // The opened file keeps the old contents even after this machine's file changed.
    let changed = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_file("/Users/t/.zshrc", "export A=2\n")
        .with_file("/x/repot.json", &text);
    let (opened, _) = Session::open(&changed, Path::new("/x/repot.json")).unwrap();
    let selection = Selection {
        items: all.iter().map(|s| s.to_string()).collect(),
        reveal_secret_lines: vec![],
    };
    let zshrc_of = |session: &Session| {
        session
            .preview(&changed, &selection, mac(), Format::Script, meta())
            .unwrap()
            .files
            .into_iter()
            .find(|f| f.path.ends_with(".zshrc"))
            .unwrap()
            .contents
    };
    assert_eq!(zshrc_of(&opened), "export A=1\n");
    assert_eq!(
        zshrc_of(&Session::from_scan(Os::MacOs, scan_of(items))),
        "export A=2\n"
    );
    // A target that cannot exist is refused.
    let bad = Target {
        os: Os::MacOs,
        packages: Some(PackageManager::Winget),
    };
    assert!(
        opened
            .preview(&changed, &selection, bad, Format::Script, meta())
            .is_err()
    );
}
