use repot_core::{
    env::FakeEnv,
    model::*,
    os::Os,
    providers::brew::Brew,
    scan::{Provider, ProviderError},
};
fn brew_env(info: &str) -> FakeEnv {
    FakeEnv::new(Os::MacOs, "/Users/t")
        .with_command(
            "brew",
            &["leaves", "--installed-on-request"],
            include_str!("fixtures/brew/leaves.txt"),
        )
        .with_command(
            "brew",
            &["list", "--cask"],
            include_str!("fixtures/brew/casks.txt"),
        )
        .with_command("brew", &["tap"], include_str!("fixtures/brew/taps.txt"))
        .with_command("brew", &["info", "--json=v2", "--installed"], info)
}
#[test]
fn scans_top_level_formulae_casks_and_taps() {
    let items = Brew
        .scan(&brew_env(include_str!("fixtures/brew/info.json")))
        .unwrap();
    let ids: Vec<_> = items.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "brew:formula:ripgrep",
            "brew:formula:jq",
            "brew:cask:visual-studio-code",
            "brew:tap:oven-sh/bun"
        ]
    );
    let rg = &items[0];
    assert_eq!(rg.version.as_deref(), Some("14.1.1"));
    assert_eq!(
        rg.description.as_deref(),
        Some("Search tool like grep and The Silver Searcher")
    );
    assert_eq!(rg.portability, Portability::SameOs);
    assert_eq!(
        rg.details,
        ItemDetails::Package {
            manager: PackageManager::Brew,
            kind: PackageKind::Formula
        }
    );
    assert!(!ids.contains(&"brew:formula:pcre2"));
}

#[test]
fn invalid_info_json_fails_whole_provider() {
    assert!(matches!(
        Brew.scan(&brew_env("{not json")),
        Err(ProviderError::Parse { .. })
    ));
}
