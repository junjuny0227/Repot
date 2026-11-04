use clap::Parser;
use repot_cli::{args::Cli, run};
use repot_core::{FakeEnv, export::ExportMeta, os::Os, scan::ScanResult};
use std::sync::Arc;

/// A Mac with two Homebrew packages, one npm global and a .zshrc holding a token.
fn mac(with_typescript: bool) -> FakeEnv {
    let globals = if with_typescript {
        r#"{"dependencies":{"typescript":{"version":"5.6.3"}}}"#
    } else {
        r#"{"dependencies":{}}"#
    };
    FakeEnv::new(Os::MacOs, "/Users/demo")
        .with_command("brew", &["leaves", "--installed-on-request"], "ripgrep\n")
        .with_command("brew", &["list", "--cask"], "rectangle\n")
        .with_command("brew", &["tap"], "")
        .with_command(
            "brew",
            &["info", "--json=v2", "--installed"],
            r#"{"formulae":[{"name":"ripgrep","desc":"Search files quickly","installed":[{"version":"14.1.1"}]}],"casks":[{"token":"rectangle","desc":"Move and resize windows","installed":"0.85"}]}"#,
        )
        .with_command("npm", &["ls", "-g", "--depth=0", "--json"], globals)
        .with_command("npm", &["root", "-g"], "/opt/npm")
        .with_file(
            "/Users/demo/.zshrc",
            "export PATH=$HOME/bin:$PATH\nexport GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz0123456789\n",
        )
}

struct Output {
    code: i32,
    out: String,
    err: String,
}
fn repot(env: FakeEnv, args: &[&str]) -> Output {
    let cli = Cli::try_parse_from(std::iter::once("repot").chain(args.iter().copied())).unwrap();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let mut ctx = run::Context {
        env: Arc::new(env),
        this_os: Os::MacOs,
        meta: ExportMeta {
            repot_version: "0.7.0".into(),
            date: "2026-11-04".into(),
            generated_at: "2026-11-04T00:00:00Z".into(),
        },
        on_progress: &|_| {},
        out: &mut out,
        err: &mut err,
    };
    let code = run::run(cli, &mut ctx);
    Output {
        code,
        out: String::from_utf8(out).unwrap(),
        err: String::from_utf8(err).unwrap(),
    }
}
fn json(o: &Output) -> serde_json::Value {
    serde_json::from_str(&o.out).unwrap()
}

#[test]
fn scan_lists_items_with_their_ids() {
    let o = repot(mac(true), &["scan"]);
    assert_eq!(o.code, 0, "{}", o.err);
    assert!(o.out.starts_with("macOS: "), "{}", o.out);
    assert!(o.out.contains("\nPackages (2)\n"), "{}", o.out);
    assert!(o.out.contains("ripgrep 14.1.1"), "{}", o.out);
    assert!(o.out.contains("brew:formula:ripgrep"), "{}", o.out);
    assert!(o.out.contains("Not installed: "), "{}", o.out);
    let scan: ScanResult =
        serde_json::from_str(&repot(mac(true), &["scan", "--json"]).out).unwrap();
    assert!(scan.items.iter().any(|i| i.id == "brew:cask:rectangle"));
}

#[test]
fn dry_run_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().to_str().unwrap();
    let o = repot(mac(true), &["export", "--dry-run", "--out", out]);
    assert_eq!(o.code, 0, "{}", o.err);
    assert!(o.out.contains("Would write:\n"), "{}", o.out);
    assert!(o.out.contains("  restore.sh\n"), "{}", o.out);
    assert!(o.out.contains("  repot.json\n"), "{}", o.out);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    // Secrets are always masked, and the user is told where.
    assert!(
        o.err.contains("note: masked secrets in .zshrc (line 2)"),
        "{}",
        o.err
    );
    let missing = dir.path().join("missing");
    let o = repot(mac(true), &["export", "--out", missing.to_str().unwrap()]);
    assert_eq!(o.code, 1);
    assert_eq!(
        o.err,
        format!("error: {} is not a folder\n", missing.display())
    );
    assert!(!missing.exists());
}

#[test]
fn export_writes_one_folder_and_says_what_to_run() {
    let dir = tempfile::tempdir().unwrap();
    let o = repot(
        mac(true),
        &["export", "--out", dir.path().to_str().unwrap()],
    );
    assert_eq!(o.code, 0, "{}", o.err);
    let folder = dir.path().join("repot-export-2026-11-04");
    assert!(
        o.out.contains(&format!("Wrote {}", folder.display())),
        "{}",
        o.out
    );
    assert!(o.out.contains("run `bash restore.sh`"), "{}", o.out);
    let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
    assert_eq!(entries.len(), 1);
    let zshrc = std::fs::read_to_string(folder.join("files/home/.zshrc")).unwrap();
    assert!(!zshrc.contains("ghp_"), "{zshrc}");
}

#[test]
fn destination_and_packages_are_checked() {
    let o = repot(
        mac(true),
        &[
            "export",
            "--dry-run",
            "--json",
            "--to",
            "windows",
            "--packages",
            "scoop",
        ],
    );
    assert_eq!(o.code, 0, "{}", o.err);
    let v = json(&o);
    assert_eq!(v["target"]["packages"], "scoop");
    assert!(v["folder"].is_null());
    // The Mac-only cask is not selected for Windows.
    let selected = v["selected"].as_array().unwrap();
    assert!(!selected.iter().any(|id| id == "brew:cask:rectangle"));
    let o = repot(mac(true), &["export", "--dry-run", "--packages", "winget"]);
    assert_eq!(o.code, 1);
    assert_eq!(o.err, "error: --packages winget cannot be used for macOS\n");
}

#[test]
fn include_and_exclude_take_known_ids_only() {
    let o = repot(
        mac(true),
        &[
            "export",
            "--dry-run",
            "--json",
            "--exclude",
            "brew:cask:rectangle",
        ],
    );
    assert_eq!(o.code, 0, "{}", o.err);
    let selected = json(&o)["selected"].as_array().unwrap().clone();
    assert!(selected.iter().any(|id| id == "brew:formula:ripgrep"));
    assert!(!selected.iter().any(|id| id == "brew:cask:rectangle"));
    let o = repot(
        mac(true),
        &["export", "--dry-run", "--include", "brew:formula:nope"],
    );
    assert_eq!(o.code, 1);
    assert!(
        o.err
            .starts_with("error: no item with id brew:formula:nope"),
        "{}",
        o.err
    );
}

/// Exports this fake Mac for real and returns the written repot.json.
fn exported_file() -> String {
    let dir = tempfile::tempdir().unwrap();
    let o = repot(
        mac(true),
        &["export", "--out", dir.path().to_str().unwrap()],
    );
    assert_eq!(o.code, 0, "{}", o.err);
    std::fs::read_to_string(dir.path().join("repot-export-2026-11-04/repot.json")).unwrap()
}

#[test]
fn compare_and_only_missing_use_this_machine() {
    let file = exported_file();
    // Same machine: everything is already here.
    let o = repot(
        mac(true).with_file("/x/repot.json", &file),
        &["compare", "/x/repot.json"],
    );
    assert_eq!(o.code, 0, "{}", o.err);
    assert!(o.out.contains("\nDifferent here: 0\n"), "{}", o.out);
    assert!(o.out.contains("\nMissing: 0\n"), "{}", o.out);
    // A machine without typescript: only it is missing, and only it is exported.
    let newer = || mac(false).with_file("/x/repot.json", &file);
    let o = repot(newer(), &["compare", "/x/repot.json"]);
    assert!(
        o.out.contains("\nMissing: 1\n  typescript 5.6.3"),
        "{}",
        o.out
    );
    let v = json(&repot(newer(), &["compare", "--json", "/x/repot.json"]));
    assert_eq!(v["npm:typescript"], "missing");
    assert_eq!(v["brew:formula:ripgrep"], "present");
    let o = repot(
        newer(),
        &[
            "export",
            "/x/repot.json",
            "--only-missing",
            "--dry-run",
            "--json",
        ],
    );
    assert_eq!(o.code, 0, "{}", o.err);
    assert_eq!(json(&o)["selected"], serde_json::json!(["npm:typescript"]));
    // Nothing missing is a result, not an error.
    let o = repot(
        mac(true).with_file("/x/repot.json", &file),
        &["export", "/x/repot.json", "--only-missing", "--dry-run"],
    );
    assert_eq!(o.code, 0, "{}", o.err);
    assert_eq!(
        o.out,
        "This machine already has everything in the file; nothing to export.\n"
    );
}

#[test]
fn only_missing_needs_a_file_and_this_machines_os() {
    let o = repot(mac(true), &["export", "--only-missing", "--dry-run"]);
    assert_eq!(o.code, 1);
    assert!(o.err.contains("give the file"), "{}", o.err);
    let env = mac(true).with_file("/x/repot.json", &exported_file());
    let o = repot(
        env,
        &[
            "export",
            "/x/repot.json",
            "--only-missing",
            "--to",
            "windows",
            "--dry-run",
        ],
    );
    assert_eq!(o.code, 1);
    assert!(o.err.contains("the destination must be macOS"), "{}", o.err);
}

#[test]
fn unreadable_and_foreign_files_fail_with_a_reason() {
    let o = repot(mac(true), &["compare", "/x/none.json"]);
    assert_eq!(
        (o.code, o.err.as_str()),
        (1, "error: could not read /x/none.json\n")
    );
    let env = mac(true).with_file("/x/a.json", r#"{"hello":"world"}"#);
    let o = repot(env, &["export", "/x/a.json", "--dry-run"]);
    assert_eq!(
        (o.code, o.err.as_str()),
        (1, "error: /x/a.json is not a Repot file\n")
    );
}
