use repot_core::{
    env::FakeEnv,
    model::*,
    os::{Os, Shell},
    providers::shell::ShellProvider,
    scan::Provider,
};

// Mimic the real filesystem's ENOTDIR when traversing a regular theme file.
struct StrictEnv(FakeEnv);
impl repot_core::env::Env for StrictEnv {
    fn temp_file(&self, name: &str) -> std::path::PathBuf {
        self.0.temp_file(name)
    }
    fn remove_file(&self, path: &std::path::Path) {
        self.0.remove_file(path);
    }
    fn read_link(&self, path: &std::path::Path) -> Option<std::path::PathBuf> {
        self.0.read_link(path)
    }
    fn os(&self) -> Os {
        self.0.os()
    }
    fn home_dir(&self) -> std::path::PathBuf {
        self.0.home_dir()
    }
    fn var(&self, n: &str) -> Option<String> {
        self.0.var(n)
    }
    fn which(&self, n: &str) -> bool {
        self.0.which(n)
    }
    fn run(
        &self,
        p: &str,
        a: &[&str],
    ) -> Result<repot_core::env::CommandOutput, repot_core::env::EnvError> {
        self.0.run(p, a)
    }
    fn read_file(&self, p: &std::path::Path) -> Result<String, repot_core::env::EnvError> {
        if p.to_string_lossy().contains("example.zsh-theme/") {
            return Err(repot_core::env::EnvError::Io("Not a directory".into()));
        }
        self.0.read_file(p)
    }
    fn list_dir(
        &self,
        p: &std::path::Path,
    ) -> Result<Vec<std::path::PathBuf>, repot_core::env::EnvError> {
        self.0.list_dir(p)
    }
    fn exists(&self, p: &std::path::Path) -> bool {
        self.0.exists(p)
    }
}
#[test]
fn ignores_regular_theme_files() {
    let env = StrictEnv(
        FakeEnv::new(Os::MacOs, "/Users/t")
            .with_file(
                "/Users/t/.oh-my-zsh/custom/themes/example.zsh-theme",
                "# local theme",
            )
            .with_file("/Users/t/.zshrc", ""),
    );
    assert_eq!(ShellProvider.scan(&env).unwrap().len(), 1);
}
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
