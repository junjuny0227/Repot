use super::{
    common::{ConfigFileSpec, config_file_item},
    git::entries,
};
use crate::{
    catalog,
    env::{Env, EnvError},
    model::*,
    os::{Os, Shell},
    scan::{Provider, ProviderError},
};
pub struct ShellProvider;
fn files(env: &dyn Env) -> Vec<(&'static str, Portability, &'static str)> {
    let mut files = vec![
        (".zshrc", Portability::SameShell(Shell::Zsh), "file:zshrc"),
        (
            ".zprofile",
            Portability::SameShell(Shell::Zsh),
            "file:zprofile",
        ),
        (
            ".bashrc",
            Portability::SameShell(Shell::Bash),
            "file:bashrc",
        ),
        (
            ".bash_profile",
            Portability::SameShell(Shell::Bash),
            "file:bash-profile",
        ),
        (
            ".profile",
            Portability::SameShell(Shell::Bash),
            "file:profile",
        ),
        (
            ".config/fish/config.fish",
            Portability::SameShell(Shell::Fish),
            "file:fish-config",
        ),
        (".config/starship.toml", Portability::Any, "file:starship"),
    ];
    for rel in if env.os() == Os::Windows {
        vec![
            "Documents/PowerShell/Microsoft.PowerShell_profile.ps1",
            "Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1",
        ]
    } else {
        vec![".config/powershell/Microsoft.PowerShell_profile.ps1"]
    } {
        files.push((
            rel,
            Portability::SameShell(Shell::PowerShell),
            "file:powershell-profile",
        ));
    }
    files
}
impl Provider for ShellProvider {
    fn id(&self) -> &'static str {
        "shell"
    }
    fn category(&self) -> Category {
        Category::Shell
    }
    fn detect(&self, env: &dyn Env) -> bool {
        files(env)
            .iter()
            .any(|(p, _, _)| env.exists(&env.home_dir().join(p)))
            || ["plugins", "themes"]
                .iter()
                .any(|k| env.exists(&env.home_dir().join(format!(".oh-my-zsh/custom/{k}"))))
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let home = env.home_dir();
        let mut items = vec![];
        for (rel, portability, catalog_key) in files(env) {
            let target = if catalog_key == "file:powershell-profile" {
                TargetPath::PowerShellProfile
            } else {
                TargetPath::Home(rel.into())
            };
            if let Some(item) = config_file_item(
                env,
                ConfigFileSpec {
                    provider: "shell",
                    category: self.category(),
                    id: format!("shell:file:{rel}"),
                    name: rel.into(),
                    path: home.join(rel),
                    target,
                    portability,
                    catalog_key,
                },
                |_| vec![],
            ) {
                items.push(item);
            }
        }
        for kind in ["plugins", "themes"] {
            let base = home.join(format!(".oh-my-zsh/custom/{kind}"));
            let mut paths = match env.list_dir(&base) {
                Ok(p) => p,
                Err(EnvError::NotFound) => continue,
                Err(e) => return Err(e.into()),
            };
            paths.sort();
            for path in paths {
                let Some(name) = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .filter(|n| *n != "example")
                else {
                    continue;
                };
                let config = path.join(".git/config");
                let content = match env.read_file(&config) {
                    Ok(s) => s,
                    Err(EnvError::NotFound | EnvError::PermissionDenied) => continue,
                    Err(e) => return Err(e.into()),
                };
                let Some((_, _, _, url)) = entries(&content)
                    .into_iter()
                    .rev()
                    .find(|(_, s, k, v)| s == r#"remote "origin""# && k == "url" && !v.is_empty())
                else {
                    continue;
                };
                items.push(Item {
                    id: format!("shell:omz:{kind}/{name}"),
                    provider: "shell".into(),
                    category: self.category(),
                    name: name.into(),
                    version: None,
                    description: catalog::describe("plugin:oh-my-zsh").map(str::to_owned),
                    origin: Origin::Path(config),
                    portability: Portability::SameShell(Shell::Zsh),
                    details: ItemDetails::ShellPlugin {
                        url,
                        target: TargetPath::Home(format!(".oh-my-zsh/custom/{kind}/{name}").into()),
                    },
                    findings: vec![],
                });
            }
        }
        Ok(items)
    }
}
