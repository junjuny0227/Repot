use repot_core::{
    env::FakeEnv,
    export::*,
    os::{LinuxFamily, Os, Shell},
    providers::{git::Git, jetbrains::JetBrains, shell::ShellProvider, ssh::Ssh, vscode::VsCode},
    scan::Provider,
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
                url: None,
            },
        ),
        item(
            "brew:cask:rectangle",
            "brew",
            Portability::Package,
            ItemDetails::Package {
                manager: PackageManager::Brew,
                kind: PackageKind::Cask,
                url: None,
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

fn not_exported_ids(text: &str) -> Vec<String> {
    let value: serde_json::Value = serde_json::from_str(text).unwrap();
    value["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["notExported"] == true)
        .map(|i| i["id"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn every_export_writes_repot_json_with_exported_and_unusable_items() {
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
    // rectangle is not in the package map and .zshrc needs zsh: left out of the script with
    // warnings, but kept in the file (marked) for another destination. The unchecked
    // typescript stays out. The scan's order is kept.
    let text = repot_json(&bundle);
    assert_eq!(
        ids(text),
        vec![
            "brew:formula:ripgrep",
            "brew:cask:rectangle",
            "shell:file:.zshrc",
            "git:file:.gitconfig",
            "shell:file:.config/starship.toml"
        ]
    );
    assert_eq!(
        not_exported_ids(text),
        vec!["brew:cask:rectangle", "shell:file:.zshrc"]
    );
    assert!(!bundle.files.iter().any(|f| f.path == "files/home/.zshrc"));
    // A config file that was not exported is stored masked like an exported one.
    assert!(!text.contains("sk-synthetic") && !text.contains("ghp_synthetic"));
    let opened = parse_snapshot(text).unwrap().snapshot;
    assert_eq!(
        opened.not_exported.iter().collect::<Vec<_>>(),
        vec!["brew:cask:rectangle", "shell:file:.zshrc"]
    );
    assert!(
        opened.configs["shell:file:.zshrc"]
            .content
            .contains("<REDACTED>")
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
        .map(|i| snapshot::StoredItem {
            item: i.clone(),
            config: snap.configs.get(&i.id).cloned(),
            not_exported: snap.not_exported.contains(&i.id),
        })
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
fn scoop_bucket_urls_round_trip_and_unsafe_ones_are_skipped() {
    let bucket = |id: &str, details: serde_json::Value| {
        serde_json::json!({
            "id": id,
            "provider": "scoop",
            "category": "packageManager",
            "name": id.rsplit(':').next().unwrap(),
            "origin": { "command": "scoop export" },
            "portability": "package",
            "details": { "package": details }
        })
    };
    let repot_url = "https://github.com/junjuny0227/scoop-bucket";
    let good = bucket(
        "scoop:bucket:repot",
        serde_json::json!({ "manager": "scoop", "kind": "tap", "url": repot_url }),
    );
    // Files written before the field existed have no url.
    let old = bucket(
        "scoop:bucket:extras",
        serde_json::json!({ "manager": "scoop", "kind": "tap" }),
    );
    let mut cases = vec![good, old];
    for (n, url) in [
        "https://x.test/a b",
        "https://x.test/a';calc;'",
        "http://x.test/a",
        "C:/buckets/x",
        "-https://x.test/a",
    ]
    .iter()
    .enumerate()
    {
        cases.push(bucket(
            &format!("bad:url:{n}"),
            serde_json::json!({ "manager": "scoop", "kind": "tap", "url": url }),
        ));
    }
    // A url only belongs to a Scoop bucket.
    cases.push(bucket(
        "bad:kind:brew",
        serde_json::json!({ "manager": "brew", "kind": "tap", "url": "https://github.com/a/b" }),
    ));
    cases.push(bucket(
        "bad:kind:app",
        serde_json::json!({ "manager": "scoop", "kind": "package", "url": "https://github.com/a/b" }),
    ));
    let opened = parse_snapshot(&with_items(serde_json::Value::Array(cases))).unwrap();
    assert_eq!(opened.skipped_items, 7);
    let snap = opened.snapshot;
    assert_eq!(
        snap.items
            .iter()
            .map(|i| i.details.clone())
            .collect::<Vec<_>>(),
        vec![
            ItemDetails::Package {
                manager: PackageManager::Scoop,
                kind: PackageKind::Tap,
                url: Some(repot_url.into()),
            },
            ItemDetails::Package {
                manager: PackageManager::Scoop,
                kind: PackageKind::Tap,
                url: None,
            },
        ]
    );
    let bundle = export(
        &snap,
        &snap.items,
        &all(&snap.items, vec![]),
        Os::Windows,
        Target::default_for(Os::Windows),
    );
    let script = &bundle
        .files
        .iter()
        .find(|f| f.path == "restore.ps1")
        .unwrap()
        .contents;
    assert!(script.contains(&format!("& scoop bucket add 'repot' '{repot_url}'")));
    let written: serde_json::Value = serde_json::from_str(repot_json(&bundle)).unwrap();
    let details = written["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["details"]["package"].clone())
        .collect::<Vec<_>>();
    assert_eq!(
        details,
        vec![
            serde_json::json!({ "manager": "scoop", "kind": "tap", "url": repot_url }),
            serde_json::json!({ "manager": "scoop", "kind": "tap" }),
        ]
    );
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

#[test]
fn a_file_for_one_destination_can_be_exported_for_another() {
    let (env, items) = machine();
    let windows = export(
        &env,
        &items,
        &all(&items, vec![]),
        Os::MacOs,
        Target::default_for(Os::Windows),
    );
    let snap = parse_snapshot(repot_json(&windows)).unwrap().snapshot;
    let mac = Target::default_for(Os::MacOs);
    // Exporting for a Mac from the Windows file: the marked items are usable again.
    let chosen = all(&snap.items, vec![]);
    let bundle = export(&snap, &snap.items, &chosen, snap.source, mac);
    let script = &bundle
        .files
        .iter()
        .find(|f| f.path == "restore.sh")
        .unwrap()
        .contents;
    assert!(
        script.contains("brew install --cask 'rectangle'"),
        "{script}"
    );
    assert!(bundle.files.iter().any(|f| f.path == "files/home/.zshrc"));
    assert!(not_exported_ids(repot_json(&bundle)).is_empty());
    // Left unchecked, a marked item counts as removed by the user and leaves the new file.
    let mut without = chosen.clone();
    without.items.retain(|id| id != "brew:cask:rectangle");
    let bundle = export(&snap, &snap.items, &without, snap.source, mac);
    assert!(!ids(repot_json(&bundle)).contains(&"brew:cask:rectangle".to_owned()));
}

#[test]
fn unreadable_or_unsafe_unusable_items_are_not_stored() {
    let (env, mut items) = machine();
    // .zshrc cannot be used on Windows and cannot be read either.
    items[3].findings.push(Finding {
        line: None,
        kind: FindingKind::Unreadable,
    });
    items[1].name = "rect'angle".into();
    let bundle = export(
        &env,
        &items,
        &all(&items, vec![]),
        Os::MacOs,
        Target::default_for(Os::Windows),
    );
    assert!(not_exported_ids(repot_json(&bundle)).is_empty());
}

#[test]
fn jetbrains_items_round_trip_and_unsafe_ones_are_skipped() {
    let base: serde_json::Value = serde_json::from_str(V1_BASIC).unwrap();
    let zshrc = base["items"][6].clone();
    let mut good_file = zshrc.clone();
    good_file["id"] = "jetbrains:IntelliJIdea:file:codestyles/Default.xml".into();
    good_file["provider"] = "jetbrains".into();
    good_file["category"] = "editor".into();
    good_file["details"]["configFile"]["target"] = serde_json::json!({
        "jetBrains": { "ide": "intelliJIdea", "version": "2026.1", "rel": "codestyles/Default.xml" }
    });
    let plugin = serde_json::json!({
        "id": "jetbrains:IntelliJIdea:plugin:com.github.copilot",
        "provider": "jetbrains",
        "category": "editor",
        "name": "com.github.copilot",
        "origin": { "path": "/x" },
        "portability": "any",
        "details": { "idePlugin": { "ide": "intelliJIdea", "folder": "github-copilot-intellij" } }
    });
    let mut cases = vec![good_file.clone(), plugin.clone()];
    for (n, version) in ["2026", "../2026.1", "2026.1/../x"].iter().enumerate() {
        let mut bad = good_file.clone();
        bad["id"] = format!("bad:version:{n}").into();
        bad["details"]["configFile"]["target"]["jetBrains"]["version"] = (*version).into();
        cases.push(bad);
    }
    for folder in ["..", "a/b", "a\\b", "x';rm -rf ~;'"] {
        let mut bad = plugin.clone();
        bad["id"] = format!("bad:folder:{folder}").into();
        bad["details"]["idePlugin"]["folder"] = folder.into();
        cases.push(bad);
    }
    let opened = parse_snapshot(&with_items(serde_json::Value::Array(cases))).unwrap();
    assert_eq!(opened.skipped_items, 7);
    let snap = opened.snapshot;
    assert_eq!(snap.items.len(), 2);
    assert_eq!(
        snap.items[1].details,
        ItemDetails::IdePlugin {
            ide: JetBrainsIde::IntelliJIdea,
            folder: "github-copilot-intellij".into()
        }
    );
    // Written back the same way.
    let items: Vec<_> = snap
        .items
        .iter()
        .map(|i| snapshot::StoredItem {
            item: i.clone(),
            config: snap.configs.get(&i.id).cloned(),
            not_exported: false,
        })
        .collect();
    let text =
        snapshot::render_snapshot(&snap.created_at, &snap.repot_version, snap.source, &items);
    assert_eq!(parse_snapshot(&text).unwrap().snapshot, snap);
}

#[test]
fn rustup_components_and_targets_round_trip_and_unsafe_names_are_skipped() {
    let toolchain = |id: &str, extra: serde_json::Value| {
        let mut details =
            serde_json::json!({ "tool": "rustup", "runtime": "rust", "isDefault": true });
        details
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        serde_json::json!({
            "id": id,
            "provider": "rustup",
            "category": "runtime",
            "name": "Rust toolchain",
            "version": "stable",
            "origin": { "command": "rustup toolchain list" },
            "portability": { "mapped": "rustup" },
            "details": { "runtime": details }
        })
    };
    let cases = vec![
        // Files written before components were scanned have neither field.
        toolchain("rustup:rust:old", serde_json::json!({})),
        toolchain(
            "rustup:rust:stable",
            serde_json::json!({ "components": ["clippy", "rust-src"], "targets": ["wasm32-unknown-unknown"] }),
        ),
        toolchain(
            "bad:component",
            serde_json::json!({ "components": ["clippy; rm -rf ~"] }),
        ),
        toolchain(
            "bad:option",
            serde_json::json!({ "components": ["--force"] }),
        ),
        toolchain("bad:target", serde_json::json!({ "targets": ["../x"] })),
    ];
    let opened = parse_snapshot(&with_items(serde_json::Value::Array(cases))).unwrap();
    assert_eq!(opened.skipped_items, 3);
    let snap = opened.snapshot;
    let details = |i: usize| match &snap.items[i].details {
        ItemDetails::Runtime {
            components,
            targets,
            ..
        } => (components.clone(), targets.clone()),
        other => panic!("{other:?}"),
    };
    assert_eq!(details(0), (vec![], vec![]));
    assert_eq!(
        details(1),
        (
            vec!["clippy".to_owned(), "rust-src".to_owned()],
            vec!["wasm32-unknown-unknown".to_owned()]
        )
    );
    let items: Vec<_> = snap
        .items
        .iter()
        .map(|i| snapshot::StoredItem {
            item: i.clone(),
            config: None,
            not_exported: false,
        })
        .collect();
    let text =
        snapshot::render_snapshot(&snap.created_at, &snap.repot_version, snap.source, &items);
    // Empty lists are not written, so toolchain items without them keep the old shape.
    assert_eq!(text.matches("\"components\"").count(), 1, "{text}");
    assert_eq!(parse_snapshot(&text).unwrap().snapshot, snap);
}

#[test]
fn skips_config_files_aimed_where_repot_never_writes() {
    let base: serde_json::Value = serde_json::from_str(V1_BASIC).unwrap();
    let gitconfig = base["items"][5].clone();
    let zshrc = base["items"][6].clone();
    let settings = base["items"][7].clone();
    let plugin = base["items"][8].clone();
    let mut ssh = gitconfig.clone();
    ssh["provider"] = "ssh".into();
    ssh["category"] = "ssh".into();
    let aimed = |from: &serde_json::Value, id: &str, target: serde_json::Value| {
        let mut item = from.clone();
        item["id"] = id.into();
        item["details"]["configFile"]["target"] = target;
        item
    };
    let jetbrains = |rel: &str| serde_json::json!({ "jetBrains": { "ide": "intelliJIdea", "version": "2026.1", "rel": rel } });
    let mut ok = vec![
        aimed(
            &gitconfig,
            "ok:git:custom-ignore",
            serde_json::json!({ "home": ".gitignore_global" }),
        ),
        aimed(
            &gitconfig,
            "ok:git:xdg-config",
            serde_json::json!({ "home": ".config/git/config" }),
        ),
        aimed(
            &gitconfig,
            "ok:git:xdg-ignore",
            serde_json::json!({ "home": ".config/git/ignore" }),
        ),
        aimed(
            &zshrc,
            "ok:shell:fish",
            serde_json::json!({ "home": ".config/fish/config.fish" }),
        ),
        aimed(
            &zshrc,
            "ok:shell:pwsh",
            serde_json::json!("powerShellProfile"),
        ),
        aimed(
            &settings,
            "ok:vscode:snippet",
            serde_json::json!({ "editorUser": { "editor": "cursor", "rel": "snippets/rust.json" } }),
        ),
    ];
    ok.push(aimed(
        &ssh,
        "ok:ssh:config",
        serde_json::json!({ "home": ".ssh/config" }),
    ));
    ok.push(aimed(
        &ssh,
        "ok:ssh:included",
        serde_json::json!({ "home": ".ssh/config.d/work" }),
    ));
    let mut jb = aimed(
        &settings,
        "ok:jetbrains:font",
        jetbrains("options/editor-font.xml"),
    );
    jb["provider"] = "jetbrains".into();
    ok.push(jb.clone());
    let mut theme = plugin.clone();
    theme["id"] = "ok:shell:theme".into();
    theme["details"]["shellPlugin"]["target"] =
        serde_json::json!({ "home": ".oh-my-zsh/custom/themes/powerlevel10k" });
    ok.push(theme);

    let home = |rel: &str| serde_json::json!({ "home": rel });
    let mut bad = vec![
        // Inside home, but places no provider scans.
        aimed(&gitconfig, "bad:git:ssh", home(".ssh/authorized_keys")),
        aimed(&zshrc, "bad:shell:ssh", home(".ssh/authorized_keys")),
        aimed(
            &zshrc,
            "bad:shell:launch-agent",
            home("Library/LaunchAgents/x.plist"),
        ),
        aimed(
            &gitconfig,
            "bad:git:autostart",
            home(".config/autostart/x.desktop"),
        ),
        aimed(
            &gitconfig,
            "bad:git:autostart-ignore",
            home(".config/autostart/ignore.desktop"),
        ),
        aimed(
            &gitconfig,
            "bad:git:launch-agent-ignore",
            home("Library/LaunchAgents/ignore.plist"),
        ),
        aimed(
            &gitconfig,
            "bad:git:other-name",
            home(".config/git/credentials"),
        ),
        aimed(&zshrc, "bad:shell:zshenv", home(".zshenv")),
        // A place another provider writes.
        aimed(&gitconfig, "bad:git:zshrc", home(".zshrc")),
        aimed(&zshrc, "bad:shell:gitconfig", home(".gitconfig")),
        aimed(
            &gitconfig,
            "bad:git:pwsh",
            serde_json::json!("powerShellProfile"),
        ),
        aimed(
            &zshrc,
            "bad:shell:editor",
            settings["details"]["configFile"]["target"].clone(),
        ),
        aimed(
            &zshrc,
            "bad:shell:jetbrains",
            jetbrains("options/editor-font.xml"),
        ),
        aimed(&settings, "bad:vscode:home", home(".zshrc")),
        // `.ssh/` belongs to `ssh` items only, and only to config files there (plan 16).
        aimed(&gitconfig, "bad:git:ssh-config", home(".ssh/config")),
        aimed(&zshrc, "bad:shell:ssh-upper", home(".SSH/config")),
        aimed(&ssh, "bad:ssh:zshrc", home(".zshrc")),
        aimed(&ssh, "bad:ssh:known-hosts", home(".ssh/known_hosts")),
        aimed(
            &ssh,
            "bad:ssh:authorized-keys",
            home(".ssh/authorized_keys"),
        ),
        aimed(&ssh, "bad:ssh:key", home(".ssh/id_ed25519")),
        aimed(&ssh, "bad:ssh:pub", home(".ssh/config.d/work.pub")),
        aimed(&ssh, "bad:ssh:upper", home(".SSH/config")),
        aimed(&ssh, "bad:ssh:pwsh", serde_json::json!("powerShellProfile")),
        // Inside the editor or IDE folder, but not a file Repot reads there.
        aimed(
            &settings,
            "bad:vscode:storage",
            serde_json::json!({ "editorUser": { "editor": "vsCode", "rel": "globalStorage/state.vscdb" } }),
        ),
        aimed(
            &settings,
            "bad:vscode:nested-snippet",
            serde_json::json!({ "editorUser": { "editor": "vsCode", "rel": "snippets/a/b.json" } }),
        ),
    ];
    for (n, rel) in [
        "options/jdk.table.xml",
        "codestyles/x.txt",
        "codestyles/a/b.xml",
        "plugins/x.jar",
    ]
    .iter()
    .enumerate()
    {
        let mut item = jb.clone();
        item["id"] = format!("bad:jetbrains:{n}").into();
        item["details"]["configFile"]["target"] = jetbrains(rel);
        bad.push(item);
    }
    let mut unknown = gitconfig.clone();
    unknown["id"] = "bad:brew:gitconfig".into();
    unknown["provider"] = "brew".into();
    bad.push(unknown);
    for (n, rel) in [
        ".oh-my-zsh/custom/plugins/x/y",
        ".oh-my-zsh/custom/plugins",
        ".oh-my-zsh/plugins/x",
        ".ssh",
        "Library/LaunchAgents/x",
    ]
    .iter()
    .enumerate()
    {
        let mut item = plugin.clone();
        item["id"] = format!("bad:plugin:{n}").into();
        item["details"]["shellPlugin"]["target"] = home(rel);
        bad.push(item);
    }
    let mut git_plugin = plugin.clone();
    git_plugin["id"] = "bad:plugin:git".into();
    git_plugin["provider"] = "git".into();
    bad.push(git_plugin);

    let bad_count = bad.len();
    let ok_ids: Vec<String> = ok
        .iter()
        .map(|i| i["id"].as_str().unwrap().to_owned())
        .collect();
    let opened = parse_snapshot(&with_items(serde_json::Value::Array([ok, bad].concat()))).unwrap();
    let kept: Vec<_> = opened.snapshot.items.iter().map(|i| i.id.clone()).collect();
    assert_eq!(kept, ok_ids);
    assert_eq!(opened.skipped_items, bad_count);
}

/// Every place the providers write on a real scan, on two OSes.
fn every_scanned_place(os: Os) -> FakeEnv {
    let (home, user, jetbrains, pwsh) = match os {
        Os::Windows => (
            "C:/Users/t",
            "C:/Users/t/AppData/Roaming/Code/User",
            "C:/Users/t/AppData/Roaming/JetBrains/IntelliJIdea2026.1",
            "C:/Users/t/Documents/PowerShell/Microsoft.PowerShell_profile.ps1",
        ),
        _ => (
            "/Users/t",
            "/Users/t/Library/Application Support/Code/User",
            "/Users/t/Library/Application Support/JetBrains/IntelliJIdea2026.1",
            "/Users/t/.config/powershell/Microsoft.PowerShell_profile.ps1",
        ),
    };
    let cursor = user.replace("/Code/", "/Cursor/");
    let ignore = if os == Os::Windows {
        "~/.config/git/gitignore"
    } else {
        "~/.gitignore_global"
    };
    let mut env = FakeEnv::new(os, home)
        .with_file(
            format!("{home}/.gitconfig"),
            &format!("[core]\n\texcludesFile = {ignore}\n"),
        )
        .with_file(format!("{home}/{}", &ignore[2..]), "*.log\n")
        .with_file(format!("{home}/.config/git/config"), "[user]\n\tname = T\n")
        .with_file(pwsh, "Set-Alias g git\n")
        .with_file(
            format!("{home}/.ssh/config"),
            "Include config.d/*\nHost *\n  IdentityFile ~/.ssh/id_ed25519\n",
        )
        .with_file(format!("{home}/.ssh/config.d/work"), "Host w\n");
    for rc in [
        ".zshrc",
        ".zprofile",
        ".bashrc",
        ".bash_profile",
        ".profile",
        ".config/fish/config.fish",
        ".config/starship.toml",
    ] {
        env = env.with_file(format!("{home}/{rc}"), "# rc\n");
    }
    for (kind, name) in [
        ("plugins", "zsh-autosuggestions"),
        ("themes", "powerlevel10k"),
    ] {
        env = env.with_file(
            format!("{home}/.oh-my-zsh/custom/{kind}/{name}/.git/config"),
            &format!("[remote \"origin\"]\n\turl = https://github.com/x/{name}\n"),
        );
    }
    for dir in [user, cursor.as_str()] {
        for rel in [
            "settings.json",
            "keybindings.json",
            "snippets/rust.json",
            "snippets/all.code-snippets",
        ] {
            env = env.with_file(format!("{dir}/{rel}"), "{}");
        }
    }
    for rel in [
        "codestyles/Default.xml",
        "keymaps/Mine.xml",
        "colors/Dark.icls",
        "templates/Custom.xml",
        "options/editor-font.xml",
        "options/console-font.xml",
        "options/colors.scheme.xml",
        "options/ui.lnf.xml",
    ] {
        env = env.with_file(format!("{jetbrains}/{rel}"), "<application/>");
    }
    env
}

#[test]
fn every_place_a_real_scan_writes_loads_from_the_file() {
    for os in [Os::MacOs, Os::Windows] {
        let env = every_scanned_place(os);
        let mut items = vec![];
        for provider in [
            &Git as &dyn Provider,
            &ShellProvider,
            &VsCode,
            &JetBrains,
            &Ssh,
        ] {
            items.extend(provider.scan(&env).unwrap());
        }
        let config_files = items
            .iter()
            .filter(|i| matches!(i.details, ItemDetails::ConfigFile { .. }))
            .count();
        // 3 git, 7 rc files + a PowerShell profile, 2 editors x 4, 8 JetBrains, 2 SSH.
        assert_eq!(config_files, 3 + 8 + 8 + 8 + 2, "{os:?}");
        assert_eq!(items.len() - config_files, 2, "{os:?}: two shell plugins");
        let selection = all(&items, vec![]);
        let saved = export(&env, &items, &selection, os, Target::default_for(os));
        let text = repot_json(&saved);
        assert_eq!(ids(text).len(), items.len(), "{os:?}");
        let opened = parse_snapshot(text).unwrap();
        assert_eq!(opened.skipped_items, 0, "{os:?}");
        let snap = &opened.snapshot;
        assert_eq!(
            snap.items.iter().map(|i| &i.details).collect::<Vec<_>>(),
            items.iter().map(|i| &i.details).collect::<Vec<_>>(),
            "{os:?}"
        );
        let via_file = export(
            snap,
            &snap.items,
            &all(&snap.items, vec![]),
            snap.source,
            Target::default_for(os),
        );
        assert_eq!(saved, via_file, "{os:?}");
    }
}

#[test]
fn ssh_items_round_trip_and_export_the_same_through_the_file() {
    let config = "Host *\n  UseKeychain yes\n  IdentityFile ~/.ssh/id_ed25519\n  SetEnv DEPLOY_TOKEN=abc123\n";
    let env = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_file("/Users/t/.ssh/config", config)
        .with_file(
            "/Users/t/.ssh/id_ed25519",
            "-----BEGIN OPENSSH PRIVATE KEY-----\n",
        );
    let items = Ssh.scan(&env).unwrap();
    assert_eq!(items.len(), 1);
    let selection = all(&items, vec![]);
    let saved = export(
        &env,
        &items,
        &selection,
        Os::MacOs,
        Target::default_for(Os::MacOs),
    );
    let text = repot_json(&saved);
    assert!(text.contains("\"category\": \"ssh\""), "{text}");
    assert!(text.contains("\"keyFile\""), "{text}");
    assert!(!text.contains("abc123") && !text.contains("PRIVATE KEY"));
    let opened = parse_snapshot(text).unwrap();
    assert_eq!(opened.skipped_items, 0);
    let snap = &opened.snapshot;
    assert_eq!(snap.items, items);
    // The file keeps the macOS form.
    assert!(
        snap.configs["ssh:file:.ssh/config"]
            .content
            .contains("\n  UseKeychain yes\n")
    );
    for target in [
        Target::default_for(Os::MacOs),
        Target::default_for(Os::Linux(LinuxFamily::Debian)),
        Target::default_for(Os::Windows),
    ] {
        let direct = export(&env, &items, &selection, Os::MacOs, target);
        let via_file = export(
            snap,
            &snap.items,
            &all(&snap.items, vec![]),
            snap.source,
            target,
        );
        assert_eq!(direct, via_file, "{target:?}");
        let file = |path: &str| {
            direct
                .files
                .iter()
                .find(|f| f.path == path)
                .map(|f| f.contents.as_str())
        };
        let payload = file("files/home/.ssh/config").unwrap();
        assert_eq!(
            payload.contains("# [repot] UseKeychain yes"),
            target.os != Os::MacOs,
            "{target:?}"
        );
        // The script-format README carries the key list even without mapped packages.
        let readme = file("README.md").expect("README.md");
        assert!(
            readme.contains("## SSH keys\n\nCopy these SSH keys yourself (Repot does not export keys):\n  - ~/.ssh/id_ed25519 (files/home/.ssh/config line 3)\n"),
            "{readme}"
        );
    }
}
