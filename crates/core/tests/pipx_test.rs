use repot_core::{
    env::FakeEnv,
    os::Os,
    providers::pipx::Pipx,
    scan::{Provider, ProviderError},
};

const LIST: &str = r#"{"pipx_spec_version":"0.1","venvs":{"black":{"metadata":{"main_package":{"package":"black","package_version":"24.10.0"}}},"pre-commit":{"metadata":{"main_package":{"package":"pre-commit","package_version":"4.0.1"}}}}}"#;

#[test]
fn packages_and_summary() {
    let venvs = "/Users/t/.local/pipx/venvs";
    let env = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_command("pipx", &["list", "--json"], LIST)
        .with_command(
            "pipx",
            &["environment", "--value", "PIPX_LOCAL_VENVS"],
            &format!("{venvs}\n"),
        )
        .with_file(
            format!("{venvs}/black/lib/python3.12/site-packages/black-24.10.0.dist-info/METADATA"),
            "Metadata-Version: 2.3\nName: black\nSummary: The uncompromising code formatter.\n",
        );
    let items = Pipx.scan(&env).unwrap();
    assert_eq!(
        items
            .iter()
            .map(|i| (i.id.as_str(), i.version.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            ("pipx:black", Some("24.10.0")),
            ("pipx:pre-commit", Some("4.0.1"))
        ]
    );
    assert_eq!(
        items[0].description.as_deref(),
        Some("The uncompromising code formatter.")
    );
    assert_eq!(items[1].description, None);
}

#[test]
fn windows_layout_and_normalized_name() {
    let venvs = "C:/Users/t/pipx/venvs";
    let env = FakeEnv::new(Os::Windows, "C:/Users/t")
        .with_command("pipx", &["list", "--json"], LIST)
        .with_command(
            "pipx",
            &["environment", "--value", "PIPX_LOCAL_VENVS"],
            &format!("{venvs}\r\n"),
        )
        .with_file(
            format!("{venvs}/pre-commit/Lib/site-packages/pre_commit-4.0.1.dist-info/METADATA"),
            "Name: pre-commit\r\nSummary: A framework for managing pre-commit hooks\r\n",
        );
    let items = Pipx.scan(&env).unwrap();
    assert_eq!(
        items[1].description.as_deref(),
        Some("A framework for managing pre-commit hooks")
    );
}

#[test]
fn environment_failure_still_lists_packages() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t").with_command("pipx", &["list", "--json"], LIST);
    assert_eq!(Pipx.scan(&env).unwrap().len(), 2);
}

#[test]
fn malformed_json_fails() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t").with_command("pipx", &["list", "--json"], "{}");
    assert!(matches!(Pipx.scan(&env), Err(ProviderError::Parse { .. })));
}
