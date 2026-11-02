use repot_core::{
    env::{CommandOutput, FakeEnv},
    os::Os,
    providers::npm::Npm,
    scan::{Provider, ProviderError},
};

#[test]
fn scoped_package_and_nonzero_exit() {
    // Review Focus 2
    let env = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_command_output("npm", &["ls", "-g", "--depth=0", "--json"], Ok(CommandOutput {
            code: Some(1), stdout: include_str!("fixtures/npm/ls.json").into(), stderr: "npm ERR! extraneous".into() }))
        .with_command("npm", &["root", "-g"], "/opt/homebrew/lib/node_modules\n")
        .with_file("/opt/homebrew/lib/node_modules/typescript/package.json",
            r#"{"description":"TypeScript is a language for application scale JavaScript development"}"#)
        .with_file("/opt/homebrew/lib/node_modules/@anthropic-ai/claude-code/package.json",
            r#"{"description":"Use Claude, Anthropic's AI assistant, right from your terminal."}"#);
    let items = Npm.scan(&env).unwrap();
    let got: Vec<_> = items
        .iter()
        .map(|i| (i.id.as_str(), i.version.as_deref()))
        .collect();
    assert_eq!(
        got,
        vec![
            ("npm:@anthropic-ai/claude-code", Some("2.0.0")),
            ("npm:typescript", Some("5.6.3"))
        ]
    );
    assert_eq!(
        items[0].description.as_deref(),
        Some("Use Claude, Anthropic's AI assistant, right from your terminal.")
    );
}

#[test]
fn nonzero_exit_without_json_fails() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t").with_command_output(
        "npm",
        &["ls", "-g", "--depth=0", "--json"],
        Ok(CommandOutput {
            code: Some(1),
            stdout: String::new(),
            stderr: "boom".into(),
        }),
    );
    assert!(matches!(
        Npm.scan(&env),
        Err(ProviderError::CommandFailed { .. })
    ));
}
