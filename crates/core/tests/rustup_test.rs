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

fn extras(item: &Item) -> (Vec<String>, Vec<String>) {
    match &item.details {
        ItemDetails::Runtime {
            components,
            targets,
            ..
        } => (components.clone(), targets.clone()),
        _ => panic!("not a runtime"),
    }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_owned()).collect()
}

fn mac_env() -> FakeEnv {
    let mut env = FakeEnv::new(Os::MacOs, "/Users/t").with_command(
        "rustup",
        &["toolchain", "list"],
        include_str!("fixtures/rustup/toolchain-list.txt"),
    );
    for (toolchain, components, targets) in [
        (
            "stable-aarch64-apple-darwin",
            include_str!("fixtures/rustup/components-stable.txt"),
            include_str!("fixtures/rustup/targets-stable.txt"),
        ),
        (
            "1.99.0-aarch64-apple-darwin",
            include_str!("fixtures/rustup/components-1.99.0.txt"),
            include_str!("fixtures/rustup/targets-1.99.0.txt"),
        ),
    ] {
        env = env
            .with_command(
                "rustup",
                &["component", "list", "--installed", "--toolchain", toolchain],
                components,
            )
            .with_command(
                "rustup",
                &["target", "list", "--installed", "--toolchain", toolchain],
                targets,
            );
    }
    env
}

#[test]
fn records_components_and_targets_without_the_host() {
    let items = Rustup.scan(&mac_env()).unwrap();
    // The host suffix is dropped (the destination has its own host), rust-std is the
    // target list's twin, and the host target comes with every toolchain.
    assert_eq!(
        extras(&items[0]),
        (
            strings(&[
                "cargo",
                "clippy",
                "rust-docs",
                "rust-src",
                "rustc",
                "rustfmt"
            ]),
            strings(&["x86_64-pc-windows-msvc"]),
        )
    );
    assert_eq!(
        extras(&items[1]),
        (strings(&["cargo", "clippy", "rustc", "rustfmt"]), vec![])
    );
}

#[test]
fn skips_names_that_are_not_plain_rustup_names() {
    let toolchain = "stable-x86_64-unknown-linux-gnu";
    let env = FakeEnv::new(Os::Linux(repot_core::os::LinuxFamily::Debian), "/home/t")
        .with_command("rustup", &["toolchain", "list"], "stable-x86_64-unknown-linux-gnu (default)\n")
        .with_command(
            "rustup",
            &["component", "list", "--installed", "--toolchain", toolchain],
            "rustc-x86_64-unknown-linux-gnu\n--evil\nrust src\n$(touch x)\nrust-analyzer-x86_64-unknown-linux-gnu\n",
        )
        .with_command(
            "rustup",
            &["target", "list", "--installed", "--toolchain", toolchain],
            "x86_64-unknown-linux-gnu\nthumbv8m.main-none-eabi\nWasm32;rm\n",
        );
    let items = Rustup.scan(&env).unwrap();
    assert_eq!(
        extras(&items[0]),
        (
            strings(&["rust-analyzer", "rustc"]),
            strings(&["thumbv8m.main-none-eabi"]),
        )
    );
}

#[test]
fn a_failing_component_list_keeps_the_toolchain() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t").with_command(
        "rustup",
        &["toolchain", "list"],
        "stable-aarch64-apple-darwin (default)\n",
    );
    let items = Rustup.scan(&env).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(extras(&items[0]), (vec![], vec![]));
}

#[test]
fn rustup_names() {
    use repot_core::providers::rustup::is_rustup_name;
    for ok in [
        "rust-src",
        "llvm-tools",
        "wasm32-unknown-unknown",
        "thumbv8m.main-none-eabi",
        "x86_64-pc-windows-msvc",
    ] {
        assert!(is_rustup_name(ok), "{ok}");
    }
    for bad in [
        "",
        "-c",
        ".x",
        "Rust",
        "a b",
        "a;b",
        "a/b",
        "a'b",
        &"a".repeat(65),
    ] {
        assert!(!is_rustup_name(bad), "{bad}");
    }
}
