use repot_core::{Env, FakeEnv, PROVIDER_TIMEOUT, default_providers, os::Os, scan};
use std::sync::Arc;

#[test]
fn default_provider_ids_are_ordered() {
    let ids: Vec<_> = default_providers().iter().map(|p| p.id()).collect();
    assert_eq!(
        ids,
        vec![
            "brew",
            "winget",
            "scoop",
            "choco",
            "apt",
            "dnf",
            "pacman",
            "nvm",
            "nvm-windows",
            "fnm",
            "volta",
            "pyenv",
            "rustup",
            "sdkman",
            "npm",
            "pnpm",
            "pipx",
            "cargo",
            "vscode",
            "git",
            "shell"
        ]
    );
    let unique: std::collections::BTreeSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len());
}

#[test]
fn full_scan_on_fake_machine_is_serializable() {
    let env: Arc<dyn Env> =
        Arc::new(FakeEnv::new(Os::MacOs, "/Users/t").with_file("/Users/t/.zshrc", "alias g=git\n"));
    let r = scan(env, default_providers(), PROVIDER_TIMEOUT, &|_| {});
    assert_eq!(r.items.len(), 1);
    let json = serde_json::to_value(&r).unwrap();
    assert_eq!(json["items"][0]["id"], "shell:file:.zshrc");
    assert_eq!(json["providers"][0]["status"], "notDetected");
}
