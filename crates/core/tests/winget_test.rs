use repot_core::{
    env::{CommandOutput, FakeEnv},
    os::Os,
    providers::winget::Winget,
    scan::Provider,
};
const TMP: &str = "/tmp/repot-fake/winget-export.json";
fn args() -> Vec<&'static str> {
    vec![
        "export",
        "-o",
        TMP,
        "--include-versions",
        "--accept-source-agreements",
        "--disable-interactivity",
    ]
}

#[test]
fn parses_winget_source_only() {
    let env = FakeEnv::new(Os::Windows, "C:/Users/t")
        .with_command("winget", &args(), "")
        .with_file(TMP, include_str!("fixtures/winget/export.json"));
    let items = Winget.scan(&env).unwrap();
    assert_eq!(
        items
            .iter()
            .map(|i| (i.id.as_str(), i.version.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            ("winget:Git.Git", Some("2.47.0")),
            ("winget:Microsoft.VisualStudioCode", Some("1.95.0"))
        ]
    );
}

#[test]
fn nonzero_exit_with_valid_file() {
    // Review Focus 1
    let env = FakeEnv::new(Os::Windows, "C:/Users/t")
        .with_command_output(
            "winget",
            &args(),
            Ok(CommandOutput {
                code: Some(-1978335226),
                stdout: String::new(),
                stderr: String::new(),
            }),
        )
        .with_file(TMP, include_str!("fixtures/winget/export.json"));
    assert_eq!(Winget.scan(&env).unwrap().len(), 2);
}

#[test]
fn not_detected_on_macos() {
    assert!(!Winget.detect(&FakeEnv::new(Os::MacOs, "/Users/t").with_program("winget")));
}
