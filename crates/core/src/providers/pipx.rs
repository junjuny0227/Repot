use super::common::{global_item, run_ok};
use crate::{
    env::Env,
    model::*,
    os::Os,
    scan::{Provider, ProviderError},
};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

const COMMAND: &str = "pipx list --json";

#[derive(Deserialize)]
struct Listing {
    venvs: BTreeMap<String, Venv>,
}
#[derive(Deserialize)]
struct Venv {
    metadata: Metadata,
}
#[derive(Deserialize)]
struct Metadata {
    main_package: MainPackage,
}
#[derive(Deserialize)]
struct MainPackage {
    package: String,
    package_version: String,
}

pub struct Pipx;

fn normalize(name: &str) -> String {
    name.to_lowercase().replace(['-', '.'], "_")
}

fn site_packages(env: &dyn Env, venv: &Path) -> Vec<PathBuf> {
    if env.os() == Os::Windows {
        return vec![venv.join("Lib").join("site-packages")];
    }
    env.list_dir(&venv.join("lib"))
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.join("site-packages"))
        .collect()
}

fn summary(env: &dyn Env, venvs: &Path, venv: &str, package: &str) -> Option<String> {
    let prefix = format!("{}-", normalize(package));
    site_packages(env, &venvs.join(venv))
        .into_iter()
        .flat_map(|dir| env.list_dir(&dir).unwrap_or_default())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_lowercase())
                .is_some_and(|n| n.starts_with(&prefix) && n.ends_with(".dist-info"))
        })
        .find_map(|dist_info| {
            let metadata = env.read_file(&dist_info.join("METADATA")).ok()?;
            metadata.lines().find_map(|l| {
                l.strip_prefix("Summary: ")
                    .map(|s| s.trim().to_owned())
                    .filter(|s| !s.is_empty())
            })
        })
}

impl Provider for Pipx {
    fn id(&self) -> &'static str {
        "pipx"
    }
    fn category(&self) -> Category {
        Category::GlobalPackage
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.which("pipx")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let out = run_ok(env, "pipx", &["list", "--json"])?;
        let listing: Listing = serde_json::from_str(&out).map_err(|e| ProviderError::Parse {
            command: COMMAND.into(),
            message: e.to_string(),
        })?;
        // Descriptions are optional: without the venvs folder, items have none.
        let venvs = run_ok(env, "pipx", &["environment", "--value", "PIPX_LOCAL_VENVS"])
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        let mut items: Vec<Item> = listing
            .venvs
            .into_iter()
            .map(|(venv, v)| {
                let package = v.metadata.main_package;
                let description = venvs
                    .as_deref()
                    .and_then(|root| summary(env, root, &venv, &package.package));
                global_item(
                    "pipx",
                    GlobalManager::Pipx,
                    package.package,
                    Some(package.package_version),
                    description,
                    COMMAND,
                )
            })
            .collect();
        items.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(items)
    }
}
