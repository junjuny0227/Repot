use repot_core::{FakeEnv, PROVIDER_TIMEOUT, default_providers, scan};
use repot_core::{availability::availability_map, os::Os};
use std::sync::Arc;
fn main() {
    let env = FakeEnv::new(Os::MacOs, "/Users/demo")
        .with_command("brew", &["leaves", "--installed-on-request"], "ripgrep\n")
        .with_command("brew", &["list", "--cask"], "rectangle\n")
        .with_command("brew", &["tap"], "")
        .with_command("brew", &["info", "--json=v2", "--installed"], r#"{"formulae":[{"name":"ripgrep","desc":"Search files quickly","installed":[{"version":"14.1.1"}]}],"casks":[{"token":"rectangle","desc":"Move and resize windows","installed":"0.85"}]}"#)
        .with_command("npm", &["ls", "-g", "--depth=0", "--json"], r#"{"dependencies":{"typescript":{"version":"5.6.3"}}}"#)
        .with_command("npm", &["root", "-g"], "/opt/npm")
        .with_file("/opt/npm/typescript/package.json", r#"{"description":"Typed JavaScript tooling"}"#)
        .with_file("/Users/demo/.zshrc", "export DEMO_TOKEN=synthetic-fixture-value\n")
        .with_file("/Users/demo/.gitconfig", "[user]\n  name = Demo\n")
        .with_program("rustup")
        .with_command("rustup", &["toolchain", "list"], "stable-aarch64-apple-darwin (default)\n")
        .with_command(
            "rustup",
            &["component", "list", "--installed", "--toolchain", "stable-aarch64-apple-darwin"],
            "cargo-aarch64-apple-darwin\nclippy-aarch64-apple-darwin\nrust-src\nrust-std-aarch64-apple-darwin\nrust-std-wasm32-unknown-unknown\nrustc-aarch64-apple-darwin\n",
        )
        .with_command(
            "rustup",
            &["target", "list", "--installed", "--toolchain", "stable-aarch64-apple-darwin"],
            "aarch64-apple-darwin\nwasm32-unknown-unknown\n",
        );
    let scan = scan(
        Arc::new(env),
        default_providers(),
        PROVIDER_TIMEOUT,
        &|_| {},
    );
    let output = serde_json::json!({
        "availability": {
            "macOs": availability_map(&scan.items, Os::MacOs, repot_core::Target::default_for(Os::MacOs)),
            "windows": availability_map(&scan.items, Os::MacOs, repot_core::Target::default_for(Os::Windows)),
        },
        "scan": scan,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
