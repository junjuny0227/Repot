use super::common::run_ok;
use crate::{
    env::Env,
    model::*,
    scan::{Provider, ProviderError},
};
use serde::Deserialize;
use std::collections::BTreeMap;
#[derive(Deserialize)]
struct Listing {
    #[serde(default)]
    dependencies: BTreeMap<String, Package>,
}
#[derive(Deserialize)]
struct Package {
    version: String,
}
pub struct Npm;
impl Provider for Npm {
    fn id(&self) -> &'static str {
        "npm"
    }
    fn category(&self) -> Category {
        Category::GlobalPackage
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.which("npm")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let out = env.run("npm", &["ls", "-g", "--depth=0", "--json"])?;
        let parsed: Listing = serde_json::from_str(&out.stdout).map_err(|e| {
            if out.success() {
                ProviderError::Parse {
                    command: "npm ls -g --depth=0 --json".into(),
                    message: e.to_string(),
                }
            } else {
                ProviderError::CommandFailed {
                    command: "npm ls -g --depth=0 --json".into(),
                    code: out.code,
                    stderr: out.stderr.clone(),
                }
            }
        })?;
        let root = run_ok(env, "npm", &["root", "-g"])?;
        Ok(parsed
            .dependencies
            .into_iter()
            .filter(|(n, _)| n != "npm" && n != "corepack")
            .map(|(name, p)| {
                let description = env
                    .read_file(
                        &std::path::Path::new(root.trim())
                            .join(&name)
                            .join("package.json"),
                    )
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    .and_then(|v| v["description"].as_str().map(str::to_owned));
                Item {
                    id: format!("npm:{name}"),
                    provider: "npm".into(),
                    category: self.category(),
                    name,
                    version: Some(p.version),
                    description,
                    origin: Origin::Command("npm ls -g --depth=0 --json".into()),
                    portability: Portability::Any,
                    details: ItemDetails::GlobalPackage {
                        manager: GlobalManager::Npm,
                    },
                    findings: vec![],
                }
            })
            .collect())
    }
}
