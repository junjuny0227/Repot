use repot_core::{
    export::*,
    os::{LinuxFamily, Os, Shell},
    snapshot::{self, SnapshotError, parse_snapshot},
    *,
};

const V1_BASIC: &str = include_str!("fixtures/snapshot/v1-basic.json");

fn meta() -> ExportMeta {
    ExportMeta {
        repot_version: "0.5.0".into(),
        date: "2026-11-04".into(),
        generated_at: "2026-11-04 10:00 UTC".into(),
    }
}
fn item(id: &str, provider: &str, portability: Portability, details: ItemDetails) -> Item {
    Item {
        id: id.into(),
        name: id.rsplit(':').next().unwrap().into(),
        provider: provider.into(),
        category: Category::Shell,
        version: None,
        description: None,
        catalog_id: None,
        origin: Origin::Command("fixture".into()),
        portability,
        details,
        findings: vec![],
    }
}
fn config(id: &str, provider: &str, source: &str, rel: &str, portability: Portability) -> Item {
    item(
        id,
        provider,
        portability,
        ItemDetails::ConfigFile {
            source: source.into(),
            target: TargetPath::Home(rel.into()),
        },
    )
}
fn machine() -> (FakeEnv, Vec<Item>) {
    let env = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_file(
            "/Users/t/.zshrc",
            "export GITHUB_TOKEN=synthetic-secret\nexport OPENAI_API_KEY=sk-synthetic000000000000000000\nalias api='curl -H ghp_synthetic0000000000000000000000000000'\n",
        )
        .with_file(
            "/Users/t/.gitconfig",
            "[user]\n\tname = T\n[credential]\n\thelper = osxkeychain\n[url \"https://user:pass@example.com/\"]\n\tinsteadOf = https://example.com/\n",
        )
        .with_file("/Users/t/.config/starship.toml", "add_newline = false\n");
    let items = vec![
        item(
            "brew:formula:ripgrep",
            "brew",
            Portability::Package,
            ItemDetails::Package {
                manager: PackageManager::Brew,
                kind: PackageKind::Formula,
            },
        ),
        item(
            "brew:cask:rectangle",
            "brew",
            Portability::Package,
            ItemDetails::Package {
                manager: PackageManager::Brew,
                kind: PackageKind::Cask,
            },
        ),
        item(
            "npm:typescript",
            "npm",
            Portability::Any,
            ItemDetails::GlobalPackage {
                manager: GlobalManager::Npm,
            },
        ),
        config(
            "shell:file:.zshrc",
            "shell",
            "/Users/t/.zshrc",
            ".zshrc",
            Portability::SameShell(Shell::Zsh),
        ),
        config(
            "git:file:.gitconfig",
            "git",
            "/Users/t/.gitconfig",
            ".gitconfig",
            Portability::Any,
        ),
        config(
            "shell:file:.config/starship.toml",
            "shell",
            "/Users/t/.config/starship.toml",
            ".config/starship.toml",
            Portability::Any,
        ),
    ];
    (env, items)
}
fn export(
    configs: &(impl ConfigSource + ?Sized),
    items: &[Item],
    selection: &Selection,
    source: Os,
    target: Target,
) -> ExportBundle {
    build_export(
        configs,
        &ExportRequest {
            items,
            selection,
            source,
            target,
            format: Format::Script,
            meta: meta(),
        },
    )
}
fn all(items: &[Item], reveal: Vec<(String, u32)>) -> Selection {
    Selection {
        items: items.iter().map(|i| i.id.clone()).collect(),
        reveal_secret_lines: reveal,
    }
}
fn repot_json(bundle: &ExportBundle) -> &str {
    &bundle
        .files
        .iter()
        .find(|f| f.path == "repot.json")
        .expect("repot.json")
        .contents
}
fn ids(text: &str) -> Vec<String> {
    let value: serde_json::Value = serde_json::from_str(text).unwrap();
    value["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn every_export_writes_repot_json_with_only_exported_items() {
    let (env, items) = machine();
    let mut selection = all(&items, vec![]);
    selection.items.retain(|id| id != "npm:typescript"); // the user unchecked it
    let bundle = export(
        &env,
        &items,
        &selection,
        Os::MacOs,
        Target::default_for(Os::Windows),
    );
    // rectangle is not in the package map and .zshrc needs zsh: both dropped with warnings.
    assert_eq!(
        ids(repot_json(&bundle)),
        vec![
            "brew:formula:ripgrep",
            "git:file:.gitconfig",
            "shell:file:.config/starship.toml"
        ]
    );
    for format in [Format::Script, Format::Manifest] {
        let bundle = build_export(
            &env,
            &ExportRequest {
                items: &items,
                selection: &selection,
                source: Os::MacOs,
                target: Target::default_for(Os::MacOs),
                format,
                meta: meta(),
            },
        );
        assert!(bundle.files.iter().any(|f| f.path == "repot.json"));
    }
}

#[test]
fn repot_json_keeps_source_form_and_masks_unrevealed_secrets() {
    let (env, items) = machine();
    let selection = all(&items, vec![("shell:file:.zshrc".into(), 1)]);
    let bundle = export(
        &env,
        &items,
        &selection,
        Os::MacOs,
        Target::default_for(Os::Windows),
    );
    let text = repot_json(&bundle);
    assert!(!text.contains("sk-synthetic"));
    assert!(!text.contains("ghp_synthetic"));
    assert!(!text.contains("user:pass"));
    // Windows payload comments out credential.helper; the Repot file keeps the macOS form.
    let payload = &bundle
        .files
        .iter()
        .find(|f| f.path == "files/home/.gitconfig")
        .unwrap()
        .contents;
    assert!(payload.contains("# [repot]"));
    assert!(text.contains("\\thelper = osxkeychain\\n"));
    assert!(!text.contains("# [repot]"));

    // Same machine: the revealed .zshrc line stays as written, the other is masked.
    let bundle = export(
        &env,
        &items,
        &selection,
        Os::MacOs,
        Target::default_for(Os::MacOs),
    );
    let opened = parse_snapshot(repot_json(&bundle)).unwrap();
    let zshrc = &opened.snapshot.configs["shell:file:.zshrc"];
    assert_eq!(
        zshrc.content,
        "export GITHUB_TOKEN=synthetic-secret\nexport OPENAI_API_KEY=<REDACTED>\nalias api='curl -H <REDACTED>'\n"
    );
    assert_eq!(zshrc.revealed, vec![1]);
    assert_eq!(
        zshrc.masked,
        vec![(2, Some("OPENAI_API_KEY".into())), (3, None)]
    );
}

#[test]
fn exporting_from_an_opened_file_matches_the_direct_export() {
    let (env, items) = machine();
    let selection = all(&items, vec![("shell:file:.zshrc".into(), 1)]);
    let targets = [
        Target::default_for(Os::MacOs),
        Target::default_for(Os::Windows),
        Target {
            os: Os::Windows,
            packages: Some(PackageManager::Scoop),
        },
        Target::default_for(Os::Linux(LinuxFamily::Debian)),
    ];
    // The file is written once (to macOS) and reused for every destination.
    let saved = export(&env, &items, &selection, Os::MacOs, targets[0]);
    let opened = parse_snapshot(repot_json(&saved)).unwrap();
    assert_eq!(opened.skipped_items, 0);
    let snap = &opened.snapshot;
    // The opened session no longer knows which lines were revealed in the UI.
    let reopened_selection = all(&snap.items, vec![]);
    for target in targets {
        let direct = export(&env, &items, &selection, Os::MacOs, target);
        let via_file = export(snap, &snap.items, &reopened_selection, snap.source, target);
        assert_eq!(direct, via_file, "target {target:?}");
    }
}

#[test]
fn reads_the_committed_v1_fixture() {
    let opened = parse_snapshot(V1_BASIC).unwrap();
    assert_eq!(opened.skipped_items, 0);
    let snap = &opened.snapshot;
    assert_eq!(snap.source, Os::MacOs);
    assert_eq!(snap.created_at, "2026-11-04T10:00Z");
    assert_eq!(snap.repot_version, "0.5.0");
    assert_eq!(snap.items.len(), 9);
    assert_eq!(snap.configs["git:file:.gitconfig"].masked, vec![(5, None)]);
    assert_eq!(
        snap.items
            .iter()
            .find(|i| i.id == "vscode:code:file:settings.json")
            .unwrap()
            .details,
        ItemDetails::ConfigFile {
            source: "/Users/me/Library/Application Support/Code/User/settings.json".into(),
            target: TargetPath::EditorUser {
                editor: Editor::VsCode,
                rel: "settings.json".into()
            }
        }
    );
    // A bare <REDACTED> is not re-detected; the recorded masked line still reaches the export.
    let bundle = export(
        snap,
        &snap.items,
        &all(&snap.items, vec![]),
        snap.source,
        Target::default_for(Os::MacOs),
    );
    let script = &bundle.files[0].contents;
    assert!(script.contains("files/home/.gitconfig line 5"), "{script}");
}

#[test]
fn rendered_file_shape_is_stable() {
    let opened = parse_snapshot(V1_BASIC).unwrap();
    let snap = opened.snapshot;
    let items: Vec<_> = snap
        .items
        .iter()
        .map(|i| (i.clone(), snap.configs.get(&i.id).cloned()))
        .collect();
    insta::assert_snapshot!(snapshot::render_snapshot(
        &snap.created_at,
        &snap.repot_version,
        snap.source,
        &items
    ));
}

fn with_items(items: serde_json::Value) -> String {
    let mut value: serde_json::Value = serde_json::from_str(V1_BASIC).unwrap();
    value["items"] = items;
    value.to_string()
}
fn first_item() -> serde_json::Value {
    serde_json::from_str::<serde_json::Value>(V1_BASIC).unwrap()["items"][0].clone()
}

#[test]
fn ignores_unknown_fields_and_skips_unknown_items() {
    let mut known = first_item();
    known["futureField"] = serde_json::json!({ "anything": true });
    let mut unknown_manager = first_item();
    unknown_manager["id"] = "nix:hello".into();
    unknown_manager["details"] =
        serde_json::json!({ "package": { "manager": "nix", "kind": "formula" } });
    let mut unknown_kind = first_item();
    unknown_kind["id"] = "x:y".into();
    unknown_kind["details"] = serde_json::json!({ "font": { "family": "Meslo" } });
    let mut text: serde_json::Value = serde_json::from_str(&with_items(serde_json::json!([
        known,
        unknown_manager,
        unknown_kind
    ])))
    .unwrap();
    text["futureTopLevel"] = 1.into();
    let opened = parse_snapshot(&text.to_string()).unwrap();
    assert_eq!(opened.snapshot.items.len(), 1);
    assert_eq!(opened.skipped_items, 2);
}

#[test]
fn rejects_files_it_cannot_read() {
    let mut newer: serde_json::Value = serde_json::from_str(V1_BASIC).unwrap();
    newer["schemaVersion"] = 2.into();
    assert_eq!(
        parse_snapshot(&newer.to_string()),
        Err(SnapshotError::NewerVersion { found: 2 })
    );
    let mut other: serde_json::Value = serde_json::from_str(V1_BASIC).unwrap();
    other["format"] = "something-else".into();
    assert_eq!(
        parse_snapshot(&other.to_string()),
        Err(SnapshotError::NotRepotFile)
    );
    assert_eq!(parse_snapshot("not json"), Err(SnapshotError::NotRepotFile));
    assert_eq!(
        parse_snapshot(r#"{"format":"repot"}"#),
        Err(SnapshotError::Invalid)
    );
    assert_eq!(
        parse_snapshot(r#"{"format":"repot","schemaVersion":1,"items":[]}"#),
        Err(SnapshotError::Invalid)
    );
    let large = format!("{}{}", V1_BASIC, " ".repeat(snapshot::MAX_BYTES));
    assert_eq!(parse_snapshot(&large), Err(SnapshotError::TooLarge));
}

#[test]
fn skips_items_that_would_write_outside_the_destination() {
    let base: serde_json::Value = serde_json::from_str(V1_BASIC).unwrap();
    let zshrc = base["items"][6].clone();
    let plugin = base["items"][8].clone();
    let mut cases = vec![];
    for rel in [
        "../.ssh/authorized_keys",
        "/etc/hosts",
        "C:/Windows/x",
        "a//b",
        "",
        "~/x",
    ] {
        let mut bad = zshrc.clone();
        bad["id"] = format!("bad:{rel}").into();
        bad["details"]["configFile"]["target"] = serde_json::json!({ "home": rel });
        cases.push(bad);
    }
    let mut editor = zshrc.clone();
    editor["id"] = "bad:editor".into();
    editor["details"]["configFile"]["target"] =
        serde_json::json!({ "editorUser": { "editor": "vsCode", "rel": "../../x" } });
    cases.push(editor);
    let mut http = plugin.clone();
    http["id"] = "bad:http".into();
    http["details"]["shellPlugin"]["url"] = "http://example.com/x".into();
    cases.push(http);
    let mut quoted = plugin.clone();
    quoted["id"] = "bad:quote".into();
    quoted["details"]["shellPlugin"]["url"] = "https://example.com/x';rm -rf ~;'".into();
    cases.push(quoted);
    let count = cases.len();
    cases.push(zshrc.clone());
    cases.push(zshrc); // duplicate id
    let opened = parse_snapshot(&with_items(serde_json::Value::Array(cases))).unwrap();
    assert_eq!(opened.snapshot.items.len(), 1);
    assert_eq!(opened.skipped_items, count + 1);
}

#[test]
fn opened_items_keep_their_findings() {
    let opened = parse_snapshot(V1_BASIC).unwrap();
    let zshrc = opened
        .snapshot
        .items
        .iter()
        .find(|i| i.id == "shell:file:.zshrc")
        .unwrap();
    assert_eq!(
        zshrc.findings,
        vec![Finding {
            line: Some(2),
            kind: FindingKind::Secret {
                name: Some("GITHUB_TOKEN".into())
            }
        }]
    );
}

#[test]
fn created_at_is_iso_utc() {
    assert_eq!(meta().created_at(), "2026-11-04T10:00Z");
}
