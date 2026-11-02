use repot_core::{env::FakeEnv, model::*, os::Os, providers::rustup::Rustup, scan::Provider};

fn is_default(item: &Item) -> bool {
    matches!(
        item.details,
        ItemDetails::Runtime {
            is_default: true,
            ..
        }
    )
}

#[test]
fn parses_old_and_new_default_markers() {
    // Review Focus 4
    for (out, default_idx) in [
        (
            "stable-aarch64-apple-darwin (default)\nnightly-2024-11-01-aarch64-apple-darwin\n1.80.0-aarch64-apple-darwin\n",
            0,
        ),
        (
            "stable-x86_64-unknown-linux-gnu\nnightly-x86_64-unknown-linux-gnu (active, default)\n",
            1,
        ),
    ] {
        let env =
            FakeEnv::new(Os::MacOs, "/Users/t").with_command("rustup", &["toolchain", "list"], out);
        let items = Rustup.scan(&env).unwrap();
        assert!(is_default(&items[default_idx]));
        assert_eq!(items.iter().filter(|i| is_default(i)).count(), 1);
    }
}

#[test]
fn channel_names() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t").with_command(
        "rustup",
        &["toolchain", "list"],
        "stable-aarch64-apple-darwin (default)\nnightly-2024-11-01-aarch64-apple-darwin\n1.80.0-aarch64-apple-darwin\nmy-local-build\n",
    );
    let items = Rustup.scan(&env).unwrap();
    assert_eq!(
        items
            .iter()
            .map(|i| i.version.clone().unwrap())
            .collect::<Vec<_>>(),
        vec!["stable", "nightly-2024-11-01", "1.80.0"]
    );
    assert_eq!(items[0].id, "rustup:rust:stable");
    assert_eq!(items[0].name, "Rust toolchain");
    assert_eq!(
        items[0].portability,
        Portability::Mapped(RuntimeTool::Rustup)
    );
}
