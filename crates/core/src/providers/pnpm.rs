use super::common::{global_item, run_ok};
use crate::{
    env::Env,
    model::*,
    scan::{Provider, ProviderError},
};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};

const COMMAND: &str = "pnpm ls -g --json";

#[derive(Deserialize)]
struct Listing {
    #[serde(default)]
    dependencies: BTreeMap<String, Package>,
}
#[derive(Deserialize)]
struct Package {
    version: String,
    path: Option<String>,
}

pub struct Pnpm;

impl Provider for Pnpm {
    fn id(&self) -> &'static str {
        "pnpm"
    }
    fn category(&self) -> Category {
        Category::GlobalPackage
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.which("pnpm")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let out = run_ok(env, "pnpm", &["ls", "-g", "--json"])?;
        let listings: Vec<Listing> =
            serde_json::from_str(&out).map_err(|e| ProviderError::Parse {
                command: COMMAND.into(),
                message: e.to_string(),
            })?;
        // Normally a single global project; if several are reported the first one wins.
        let mut packages = BTreeMap::new();
        for listing in listings {
            for (name, package) in listing.dependencies {
                packages.entry(name).or_insert(package);
            }
        }
        Ok(packages
            .into_iter()
            .map(|(name, package)| {
                let description = package
                    .path
                    .and_then(|p| env.read_file(&Path::new(&p).join("package.json")).ok())
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    .and_then(|v| v["description"].as_str().map(str::to_owned));
                global_item(
                    "pnpm",
                    GlobalManager::Pnpm,
                    name,
                    Some(package.version),
                    description,
                    COMMAND,
                )
            })
            .collect())
    }
}
