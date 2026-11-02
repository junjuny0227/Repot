use repot_core::{
    env::FakeEnv,
    model::{ItemDetails, PackageKind, PackageManager},
    os::Os,
    providers::scoop::Scoop,
    scan::{Provider, ProviderError},
};
#[test]
fn buckets_and_apps() {
    let env = FakeEnv::new(Os::Windows, "C:/Users/t")
        .with_command(
            "scoop",
            &["export"],
            include_str!("fixtures/scoop/export.json"),
        )
        .with_file(
            "C:/Users/t/scoop/buckets/main/bucket/git.json",
            r#"{"description":"Distributed version control system"}"#,
        );
    let items = Scoop.scan(&env).unwrap();
    assert_eq!(
        items.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        vec!["scoop:bucket:extras", "scoop:main/git", "scoop:extras/vlc"]
    );
    assert_eq!(
        items[1].description.as_deref(),
        Some("Distributed version control system")
    );
    assert_eq!(
        items[0].details,
        ItemDetails::Package {
            manager: PackageManager::Scoop,
            kind: PackageKind::Tap
        }
    );
}

#[test]
fn text_output_is_parse_error() {
    let env = FakeEnv::new(Os::Windows, "C:/Users/t").with_command(
        "scoop",
        &["export"],
        "git (v:2.47.0) [main]\n",
    );
    assert!(matches!(Scoop.scan(&env), Err(ProviderError::Parse { .. })));
}
