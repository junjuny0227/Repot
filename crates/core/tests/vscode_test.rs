use repot_core::{env::FakeEnv, model::*, os::Os, providers::vscode::VsCode, scan::Provider};

#[test]
fn resolves_nls_placeholder() {
    // Review Focus 4
    let ext = "/Users/t/.vscode/extensions";
    let env = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_file(
            format!("{ext}/extensions.json"),
            include_str!("fixtures/vscode/extensions.json"),
        )
        .with_file(
            format!("{ext}/esbenp.prettier-vscode-11.0.0/package.json"),
            r#"{"description":"Code formatter using prettier"}"#,
        )
        .with_file(
            format!("{ext}/ms-python.python-2024.20.0/package.json"),
            r#"{"description":"%extension.description%"}"#,
        )
        .with_file(
            format!("{ext}/ms-python.python-2024.20.0/package.nls.json"),
            r#"{"extension.description":"Python language support"}"#,
        )
        .with_file(
            "/Users/t/Library/Application Support/Code/User/settings.json",
            "{}",
        )
        .with_file(
            "/Users/t/Library/Application Support/Code/User/snippets/rust.json",
            "{}",
        );
    let items = VsCode.scan(&env).unwrap();
    let got: Vec<_> = items
        .iter()
        .map(|i| (i.id.as_str(), i.description.as_deref()))
        .collect();
    assert_eq!(
        got[0],
        (
            "vscode:code:ext:esbenp.prettier-vscode",
            Some("Code formatter using prettier")
        )
    );
    assert_eq!(
        got[1],
        (
            "vscode:code:ext:ms-python.python",
            Some("Python language support")
        )
    );
    assert_eq!(items[2].id, "vscode:code:file:settings.json");
    assert_eq!(
        items[3].details,
        ItemDetails::ConfigFile {
            source: "/Users/t/Library/Application Support/Code/User/snippets/rust.json".into(),
            target: TargetPath::EditorUser {
                editor: Editor::VsCode,
                rel: "snippets/rust.json".into()
            }
        }
    );
}

#[test]
fn cursor_on_windows_uses_appdata() {
    let env = FakeEnv::new(Os::Windows, "C:/Users/t")
        .with_var("APPDATA", "C:/Users/t/AppData/Roaming")
        .with_file(
            "C:/Users/t/AppData/Roaming/Cursor/User/keybindings.json",
            "[]",
        );
    let items = VsCode.scan(&env).unwrap();
    assert_eq!(
        items.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        vec!["vscode:cursor:file:keybindings.json"]
    );
}

#[test]
fn keeps_newest_version_when_extension_is_listed_twice() {
    // VS Code keeps the old entry in extensions.json after an update until it cleans up.
    let ext = "/Users/t/.vscode/extensions";
    let env = FakeEnv::new(Os::MacOs, "/Users/t").with_file(
        format!("{ext}/extensions.json"),
        include_str!("fixtures/vscode/extensions-duplicate.json"),
    );
    let items = VsCode.scan(&env).unwrap();
    let got: Vec<_> = items
        .iter()
        .map(|i| (i.id.as_str(), i.version.as_deref()))
        .collect();
    assert_eq!(
        got,
        vec![
            (
                "vscode:code:ext:github.vscode-pull-request-github",
                Some("0.174.0")
            ),
            ("vscode:code:ext:esbenp.prettier-vscode", Some("11.0.0")),
        ]
    );
}
