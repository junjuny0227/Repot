use crate::{
    env::{Env, EnvError},
    model::*,
    scan::{Provider, ProviderError},
};
use std::path::PathBuf;
pub struct Nvm;
fn root(env: &dyn Env) -> PathBuf {
    env.var("NVM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env.home_dir().join(".nvm"))
}
fn numeric(s: &str) -> Option<Vec<u64>> {
    s.split('.').map(|v| v.parse().ok()).collect()
}
impl Provider for Nvm {
    fn id(&self) -> &'static str {
        "nvm"
    }
    fn category(&self) -> Category {
        Category::Runtime
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.exists(&root(env).join("versions/node"))
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let root = root(env);
        let mut versions = vec![];
        for path in env.list_dir(&root.join("versions/node"))? {
            let Some(version) = path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_prefix('v'))
            else {
                continue;
            };
            if let Some(numbers) = numeric(version).filter(|n| n.len() == 3) {
                versions.push((numbers, version.to_owned(), path));
            }
        }
        versions.sort_by(|a, b| a.0.cmp(&b.0));
        let alias = match env.read_file(&root.join("alias/default")) {
            Ok(s) => s,
            Err(EnvError::NotFound) => String::new(),
            Err(e) => return Err(e.into()),
        };
        let prefix =
            numeric(alias.trim().trim_start_matches('v')).filter(|n| !n.is_empty() && n.len() <= 3);
        let default = prefix.and_then(|p| versions.iter().rposition(|v| v.0.starts_with(&p)));
        Ok(versions
            .into_iter()
            .enumerate()
            .map(|(i, (_, version, path))| Item {
                id: format!("nvm:node:{version}"),
                provider: "nvm".into(),
                category: self.category(),
                name: "Node.js".into(),
                version: Some(version),
                description: None,
                origin: Origin::Path(path),
                portability: Portability::Mapped(RuntimeTool::Nvm),
                details: ItemDetails::Runtime {
                    tool: RuntimeTool::Nvm,
                    runtime: "node".into(),
                    is_default: default == Some(i),
                },
                findings: vec![],
            })
            .collect())
    }
}
