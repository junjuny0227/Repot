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
    eprintln!("{}", serde_json::to_string_pretty(&result.providers).unwrap());
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
    let expected: &[&str] = match current_os() {
        Os::MacOs => &["brew", "npm", "cargo"],
        Os::Windows => &["choco", "npm", "cargo"],
        Os::Linux(_) => &["apt", "npm", "cargo"],
    };
    for id in expected {
        assert!(ok(id), "{id} should be ok on this runner");
    }
}
