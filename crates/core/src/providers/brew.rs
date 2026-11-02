use super::common::run_ok;
use crate::{
    env::Env,
    model::*,
    scan::{Provider, ProviderError},
};
use serde::Deserialize;
#[derive(Deserialize)]
struct Info {
    formulae: Vec<Formula>,
    casks: Vec<Cask>,
}
#[derive(Deserialize)]
struct Formula {
    name: String,
    desc: Option<String>,
    installed: Vec<Version>,
}
#[derive(Deserialize)]
struct Version {
    version: String,
}
#[derive(Deserialize)]
struct Cask {
    token: String,
    desc: Option<String>,
    installed: Option<String>,
}
pub struct Brew;
impl Provider for Brew {
    fn id(&self) -> &'static str {
        "brew"
    }
    fn category(&self) -> Category {
        Category::PackageManager
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.which("brew")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let leaves = run_ok(env, "brew", &["leaves", "--installed-on-request"])?;
        let casks = run_ok(env, "brew", &["list", "--cask"])?;
        let taps = run_ok(env, "brew", &["tap"])?;
        let raw = run_ok(env, "brew", &["info", "--json=v2", "--installed"])?;
        let parse = |message: String| ProviderError::Parse {
            command: "brew info --json=v2 --installed".into(),
            message,
        };
        let info: Info = serde_json::from_str(&raw).map_err(|e| parse(e.to_string()))?;
        let mut items = vec![];
        for (text, kind, label, command) in [
            (
                &leaves,
                PackageKind::Formula,
                "formula",
                "brew leaves --installed-on-request",
            ),
            (&casks, PackageKind::Cask, "cask", "brew list --cask"),
            (&taps, PackageKind::Tap, "tap", "brew tap"),
        ] {
            for name in text.lines().map(str::trim).filter(|s| !s.is_empty()) {
                if kind == PackageKind::Tap && ["homebrew/core", "homebrew/cask"].contains(&name) {
                    continue;
                }
                let (version, description) = match kind {
                    PackageKind::Formula => {
                        let f = info
                            .formulae
                            .iter()
                            .find(|f| f.name == name)
                            .ok_or_else(|| parse(format!("missing formula {name}")))?;
                        (
                            f.installed.first().map(|v| v.version.clone()),
                            f.desc.clone(),
                        )
                    }
                    PackageKind::Cask => {
                        let c = info
                            .casks
                            .iter()
                            .find(|c| c.token == name)
                            .ok_or_else(|| parse(format!("missing cask {name}")))?;
                        (c.installed.clone(), c.desc.clone())
                    }
                    _ => (None, None),
                };
                items.push(Item {
                    id: format!("brew:{label}:{name}"),
                    provider: "brew".into(),
                    category: self.category(),
                    name: name.into(),
                    version,
                    description,
                    origin: Origin::Command(command.into()),
                    portability: Portability::SameOs,
                    details: ItemDetails::Package {
                        manager: PackageManager::Brew,
                        kind,
                    },
                    findings: vec![],
                });
            }
        }
        Ok(items)
    }
}
