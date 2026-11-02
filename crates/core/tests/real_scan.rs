use repot_core::{
    Env, PROVIDER_TIMEOUT, RealEnv, default_providers,
    os::{Os, current_os},
    scan::{ProviderStatus, scan},
};
use std::sync::Arc;

#[test]
#[ignore = "runs real tools; executed in CI with --ignored"]
fn real_scan_has_no_failures() {
    let env: Arc<dyn Env> = Arc::new(RealEnv::new());
    let result = scan(env, default_providers(), PROVIDER_TIMEOUT, &|_| {});
    eprintln!(
        "{}",
        serde_json::to_string_pretty(&result.providers).unwrap()
    );
    let bad: Vec<_> = result
        .providers
        .iter()
        .filter(|p| {
            matches!(
                p.status,
                ProviderStatus::Failed { .. } | ProviderStatus::TimedOut
            )
        })
        .collect();
    assert!(bad.is_empty(), "failed providers: {bad:#?}");
    let ok = |id: &str| {
        result
            .providers
            .iter()
            .any(|p| p.id == id && matches!(p.status, ProviderStatus::Ok { .. }))
    };
    let defaults = match current_os() {
        Os::MacOs => "brew,npm,cargo",
        Os::Windows => "choco,npm,cargo",
        Os::Linux(_) => "apt,npm,cargo",
    };
    // Containers intentionally install only their native manager and Rust.
    // Windows adds winget only if the hosted image actually provides it.
    let required = std::env::var("REPOT_REQUIRED_PROVIDERS").unwrap_or_else(|_| defaults.into());
    assert!(
        !required.trim().is_empty(),
        "required providers must not be empty"
    );
    for id in required.split(',').map(str::trim) {
        assert!(ok(id), "{id} should be ok on this runner");
    }
    // Seeded packages prove a successful status is not an empty/partial parse.
    if let Ok(required_items) = std::env::var("REPOT_REQUIRED_ITEMS") {
        for id in required_items.split(',').map(str::trim) {
            assert!(
                result.items.iter().any(|item| item.id == id),
                "seeded item {id} missing from scan"
            );
        }
    }
}
