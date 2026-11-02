use super::common::{RuntimeSpec, runtime_item};
use crate::{
    env::{Env, EnvError},
    model::*,
    scan::{Provider, ProviderError},
};
use std::path::{Path, PathBuf};

pub struct Sdkman;

fn candidates_dir(env: &dyn Env) -> PathBuf {
    env.var("SDKMAN_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env.home_dir().join(".sdkman"))
        .join("candidates")
}

fn file_name(path: &Path) -> Option<&str> {
    path.file_name().and_then(|n| n.to_str())
}

/// Lists child entries; a plain file where a folder is expected has no children.
fn children(env: &dyn Env, dir: &Path) -> Result<Vec<PathBuf>, ProviderError> {
    match env.list_dir(dir) {
        Ok(entries) => Ok(entries),
        Err(EnvError::Io(_) | EnvError::NotFound) => Ok(vec![]),
        Err(e) => Err(e.into()),
    }
}

impl Provider for Sdkman {
    fn id(&self) -> &'static str {
        "sdkman"
    }
    fn category(&self) -> Category {
        Category::Runtime
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.exists(&candidates_dir(env))
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let mut items = vec![];
        let mut candidates = children(env, &candidates_dir(env))?;
        candidates.sort();
        for candidate_dir in candidates {
            let Some(candidate) = file_name(&candidate_dir).filter(|n| !n.starts_with('.')) else {
                continue;
            };
            let current = env
                .read_link(&candidate_dir.join("current"))
                .and_then(|t| file_name(&t).map(str::to_owned));
            let mut versions = children(env, &candidate_dir)?;
            versions.sort();
            for version_dir in versions {
                let Some(version) =
                    file_name(&version_dir).filter(|n| *n != "current" && !n.starts_with('.'))
                else {
                    continue;
                };
                items.push(runtime_item(RuntimeSpec {
                    provider: "sdkman",
                    tool: RuntimeTool::Sdkman,
                    runtime: candidate,
                    name: candidate,
                    version: version.to_owned(),
                    is_default: current.as_deref() == Some(version),
                    origin: Origin::Path(version_dir.clone()),
                    catalog_key: "runtime:sdkman",
                }));
            }
        }
        Ok(items)
    }
}
