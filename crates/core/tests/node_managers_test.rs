use repot_core::{
    env::{CommandOutput, FakeEnv},
    model::*,
    os::Os,
    providers::node_managers::{Fnm, NvmWindows, Volta},
    scan::{Provider, ProviderError},
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
fn nvm_windows_list() {
    let env = FakeEnv::new(Os::Windows, "C:/Users/t").with_command(
        "nvm",
        &["list"],
        "\n  * 20.11.0 (Currently using 64-bit executable)\n    18.20.4\n",
    );
    let v: Vec<_> = NvmWindows
        .scan(&env)
        .unwrap()
        .iter()
        .map(|i| (i.id.clone(), is_default(i)))
        .collect();
    assert_eq!(
        v,
        vec![
            ("nvm-windows:node:18.20.4".into(), false),
            ("nvm-windows:node:20.11.0".into(), true)
        ]
    );
}

#[test]
fn fnm_list() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t").with_command(
        "fnm",
        &["list"],
        "* v20.11.0 default\n* v18.20.4\n* system\n",
    );
    let v: Vec<_> = Fnm
        .scan(&env)
        .unwrap()
        .iter()
        .map(|i| (i.version.clone().unwrap(), is_default(i)))
        .collect();
    assert_eq!(v, vec![("18.20.4".into(), false), ("20.11.0".into(), true)]);
}

#[test]
fn volta_runtimes_only() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t").with_command(
        "volta",
        &["list", "--format", "plain"],
        "runtime node@20.11.0 (default)\nruntime node@18.20.4\npackage-manager pnpm@9.12.0\npackage typescript@5.6.3 / tsc, tsserver / node@20.11.0 npm@built-in\n",
    );
    let items = Volta.scan(&env).unwrap();
    assert_eq!(
        items.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        vec!["volta:node:18.20.4", "volta:node:20.11.0"]
    );
    assert!(is_default(&items[1]));
    assert_eq!(
        items[0].portability,
        Portability::Mapped(RuntimeTool::Volta)
    );
}

#[test]
fn detect_checks_os_for_nvm_windows() {
    let mac = FakeEnv::new(Os::MacOs, "/Users/t").with_program("nvm");
    assert!(!NvmWindows.detect(&mac));
    let win = FakeEnv::new(Os::Windows, "C:/Users/t").with_program("nvm");
    assert!(NvmWindows.detect(&win));
    assert!(Fnm.detect(&FakeEnv::new(Os::MacOs, "/Users/t").with_program("fnm")));
    assert!(!Volta.detect(&FakeEnv::new(Os::MacOs, "/Users/t")));
}

#[test]
fn failed_command_fails_provider() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t").with_command_output(
        "fnm",
        &["list"],
        Ok(CommandOutput {
            code: Some(1),
            stdout: String::new(),
            stderr: "boom".into(),
        }),
    );
    assert!(matches!(
        Fnm.scan(&env),
        Err(ProviderError::CommandFailed { .. })
    ));
}
