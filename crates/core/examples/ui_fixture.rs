use repot_core::{FakeEnv, PROVIDER_TIMEOUT, default_providers, scan};
use repot_core::{availability::availability_map, os::Os};
use std::sync::Arc;
fn main() {
    let env = FakeEnv::new(Os::MacOs, "/Users/demo")
        .with_command("brew", &["leaves", "--installed-on-request"], "ripgrep\n")
        .with_command("brew", &["list", "--cask"], "")
        .with_command("brew", &["tap"], "")
        .with_command("brew", &["info", "--json=v2", "--installed"], r#"{"formulae":[{"name":"ripgrep","desc":"Search files quickly","installed":[{"version":"14.1.1"}]}],"casks":[]}"#)
        .with_command("npm", &["ls", "-g", "--depth=0", "--json"], r#"{"dependencies":{"typescript":{"version":"5.6.3"}}}"#)
        .with_command("npm", &["root", "-g"], "/opt/npm")
        .with_file("/opt/npm/typescript/package.json", r#"{"description":"Typed JavaScript tooling"}"#)
        .with_file("/Users/demo/.zshrc", "export DEMO_TOKEN=synthetic-fixture-value\n")
        .with_file("/Users/demo/.gitconfig", "[user]\n  name = Demo\n");
    let scan = scan(
        Arc::new(env),
        default_providers(),
        PROVIDER_TIMEOUT,
        &|_| {},
    );
    let output = serde_json::json!({
        "availability": {
            "macOs": availability_map(&scan.items, Os::MacOs, Os::MacOs),
            "windows": availability_map(&scan.items, Os::MacOs, Os::Windows),
        },
        "scan": scan,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
