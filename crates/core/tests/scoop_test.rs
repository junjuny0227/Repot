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
            kind: PackageKind::Tap,
            url: None,
        }
    );
}

#[test]
fn custom_buckets_keep_their_source_url() {
    let env = FakeEnv::new(Os::Windows, "C:/Users/t").with_command(
        "scoop",
        &["export"],
        include_str!("fixtures/scoop/export-custom-bucket.json"),
    );
    let items = Scoop.scan(&env).unwrap();
    let buckets = items
        .iter()
        .filter_map(|i| match &i.details {
            ItemDetails::Package {
                kind: PackageKind::Tap,
                url,
                ..
            } => Some((i.id.as_str(), url.as_deref())),
            _ => None,
        })
        .collect::<Vec<_>>();
    // Known buckets are added by name, so their (official) URL is not kept.
    assert_eq!(
        buckets,
        vec![
            ("scoop:bucket:extras", None),
            (
                "scoop:bucket:repot",
                Some("https://github.com/junjuny0227/scoop-bucket")
            ),
            (
                "scoop:bucket:work",
                Some("git@github.com:acme/scoop-work.git")
            ),
        ]
    );
    assert!(items.iter().any(|i| i.id == "scoop:repot/repot"));
}

#[test]
fn known_bucket_names_ignore_case_and_missing_sources_are_none() {
    let env = FakeEnv::new(Os::Windows, "C:/Users/t").with_command(
        "scoop",
        &["export"],
        r#"{"buckets":[{"Name":"Nerd-Fonts","Source":"https://github.com/matthewjberger/scoop-nerd-fonts"},{"Name":"old"},{"Name":"blank","Source":""}],"apps":[]}"#,
    );
    let urls = Scoop
        .scan(&env)
        .unwrap()
        .into_iter()
        .map(|i| match i.details {
            ItemDetails::Package { url, .. } => url,
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();
    assert_eq!(urls, vec![None, None, None]);
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
