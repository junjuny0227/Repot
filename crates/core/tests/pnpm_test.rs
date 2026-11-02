use repot_core::{
    env::FakeEnv,
    model::*,
    os::Os,
    providers::pnpm::Pnpm,
    scan::{Provider, ProviderError},
};

#[test]
fn global_packages_with_descriptions() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_command(
            "pnpm",
            &["ls", "-g", "--json"],
            r#"[{"path":"/Users/t/Library/pnpm/global/5","dependencies":{"vercel":{"version":"39.1.0","path":"/Users/t/Library/pnpm/global/5/node_modules/vercel"}}}]"#,
        )
        .with_file(
            "/Users/t/Library/pnpm/global/5/node_modules/vercel/package.json",
            r#"{"description":"The command-line interface for Vercel"}"#,
        );
    let items = Pnpm.scan(&env).unwrap();
    assert_eq!(items[0].id, "pnpm:vercel");
    assert_eq!(items[0].version.as_deref(), Some("39.1.0"));
    assert_eq!(
        items[0].description.as_deref(),
        Some("The command-line interface for Vercel")
    );
    assert_eq!(items[0].portability, Portability::Any);
    assert_eq!(
        items[0].details,
        ItemDetails::GlobalPackage {
            manager: GlobalManager::Pnpm
        }
    );
}

#[test]
fn empty_global_list() {
    // Review Focus 5
    for out in ["[]", r#"[{"path":"/x"}]"#] {
        let env =
            FakeEnv::new(Os::MacOs, "/Users/t").with_command("pnpm", &["ls", "-g", "--json"], out);
        assert!(Pnpm.scan(&env).unwrap().is_empty());
    }
}

#[test]
fn malformed_json_fails() {
    let env =
        FakeEnv::new(Os::MacOs, "/Users/t").with_command("pnpm", &["ls", "-g", "--json"], "nope");
    assert!(matches!(Pnpm.scan(&env), Err(ProviderError::Parse { .. })));
}
