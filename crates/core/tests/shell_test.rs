use repot_core::{
    env::FakeEnv,
    model::*,
    os::{Os, Shell},
    providers::shell::ShellProvider,
    scan::Provider,
};

#[test]
fn unix_shell_files_and_omz_plugins() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_file(
            "/Users/t/.zshrc",
            "export GITHUB_TOKEN=ghp_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n",
        )
        .with_file("/Users/t/.config/starship.toml", "")
        .with_file(
            "/Users/t/.oh-my-zsh/custom/plugins/zsh-autosuggestions/.git/config",
            "[remote \"origin\"]\n\turl = https://github.com/zsh-users/zsh-autosuggestions\n",
        )
        .with_file(
            "/Users/t/.oh-my-zsh/custom/plugins/example/example.plugin.zsh",
            "",
        )
        .with_file(
            "/Users/t/.oh-my-zsh/custom/plugins/local-only/local.plugin.zsh",
            "",
        );
    let items = ShellProvider.scan(&env).unwrap();
    let ids: Vec<_> = items.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "shell:file:.zshrc",
            "shell:file:.config/starship.toml",
            "shell:omz:plugins/zsh-autosuggestions"
        ]
    );
    assert_eq!(items[0].portability, Portability::SameShell(Shell::Zsh));
    assert_eq!(
        items[0].findings,
        vec![Finding {
            line: Some(1),
            kind: FindingKind::Secret {
                name: Some("GITHUB_TOKEN".into())
            }
        }]
    );
    assert_eq!(items[1].portability, Portability::Any);
    assert_eq!(
        items[2].details,
        ItemDetails::ShellPlugin {
            url: "https://github.com/zsh-users/zsh-autosuggestions".into(),
            target: TargetPath::Home(".oh-my-zsh/custom/plugins/zsh-autosuggestions".into())
        }
    );
}

#[test]
fn windows_powershell_profile() {
    let env = FakeEnv::new(Os::Windows, "C:/Users/t").with_file(
        "C:/Users/t/Documents/PowerShell/Microsoft.PowerShell_profile.ps1",
        "Set-Alias g git\n",
    );
    let items = ShellProvider.scan(&env).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0].portability,
        Portability::SameShell(Shell::PowerShell)
    );
    assert!(matches!(
        items[0].details,
        ItemDetails::ConfigFile {
            target: TargetPath::PowerShellProfile,
            ..
        }
    ));
}

#[test]
fn home_with_spaces_and_hangul() {
    // Review Focus 5
    let env = FakeEnv::new(Os::MacOs, "/Users/홍 길동")
        .with_file("/Users/홍 길동/.bashrc", "alias ll='ls -l'\n");
    let items = ShellProvider.scan(&env).unwrap();
    assert_eq!(items[0].id, "shell:file:.bashrc");
    assert_eq!(
        items[0].origin,
        Origin::Path("/Users/홍 길동/.bashrc".into())
    );
}
