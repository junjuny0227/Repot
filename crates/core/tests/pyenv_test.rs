use repot_core::{
    catalog::describe, env::FakeEnv, model::*, os::Os, providers::pyenv::Pyenv, scan::Provider,
};

fn is_default(item: &Item) -> bool {
    matches!(
        item.details,
        ItemDetails::Runtime {
            is_default: true,
            ..
        }
    )
}

#[test]
fn versions_and_global() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_command("pyenv", &["versions", "--bare"], "3.11.9\n3.12.7\n")
        .with_command("pyenv", &["global"], "3.12.7\n3.11.9\n");
    let items = Pyenv.scan(&env).unwrap();
    assert_eq!(
        items
            .iter()
            .map(|i| (i.id.as_str(), is_default(i)))
            .collect::<Vec<_>>(),
        vec![
            ("pyenv:python:3.11.9", false),
            ("pyenv:python:3.12.7", true)
        ]
    );
    assert_eq!(
        items[0].portability,
        Portability::Mapped(RuntimeTool::Pyenv)
    );
    assert_eq!(items[0].name, "Python");
    assert_eq!(items[0].description.as_deref(), describe("runtime:python"));
    assert_eq!(items[0].catalog_id.as_deref(), Some("runtime:python"));
}

#[test]
fn windows_uses_pyenv_win_and_system_has_no_default() {
    let env = FakeEnv::new(Os::Windows, "C:/Users/t")
        .with_command("pyenv", &["versions", "--bare"], "3.12.7\n")
        .with_command("pyenv", &["global"], "system\n");
    let items = Pyenv.scan(&env).unwrap();
    assert_eq!(
        items[0].portability,
        Portability::Mapped(RuntimeTool::PyenvWin)
    );
    assert!(!is_default(&items[0]));
}

#[test]
fn catalog_has_new_runtime_descriptions() {
    for key in ["runtime:python", "runtime:rust", "runtime:sdkman"] {
        assert!(describe(key).is_some(), "{key}");
    }
}
