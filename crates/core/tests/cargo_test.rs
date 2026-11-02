use repot_core::{
    env::FakeEnv,
    model::*,
    os::Os,
    providers::cargo::Cargo,
    scan::{Provider, ProviderError},
};

#[test]
fn installed_crates() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t").with_command(
        "cargo",
        &["install", "--list"],
        "cargo-insta v1.41.1:\n    cargo-insta\nripgrep v14.1.1:\n    rg\nmy-tool v0.1.0 (/Users/t/src/my-tool):\n    my-tool\n",
    );
    let items = Cargo.scan(&env).unwrap();
    assert_eq!(
        items
            .iter()
            .map(|i| (i.id.as_str(), i.version.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            ("cargo:cargo-insta", Some("1.41.1")),
            ("cargo:ripgrep", Some("14.1.1")),
            ("cargo:my-tool", Some("0.1.0"))
        ]
    );
    assert_eq!(items[0].description, None);
    assert_eq!(items[0].portability, Portability::Any);
    assert_eq!(
        items[0].details,
        ItemDetails::GlobalPackage {
            manager: GlobalManager::Cargo
        }
    );
}

#[test]
fn empty_list_and_unexpected_line() {
    let empty =
        FakeEnv::new(Os::MacOs, "/Users/t").with_command("cargo", &["install", "--list"], "");
    assert!(Cargo.scan(&empty).unwrap().is_empty());
    let bad = FakeEnv::new(Os::MacOs, "/Users/t").with_command(
        "cargo",
        &["install", "--list"],
        "warning: something odd\n",
    );
    assert!(matches!(Cargo.scan(&bad), Err(ProviderError::Parse { .. })));
}
