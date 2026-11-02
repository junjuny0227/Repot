use super::common::{package_item, run_ok};
use crate::{
    env::Env,
    model::*,
    os::Os,
    scan::{Provider, ProviderError},
};
use std::collections::BTreeMap;

const ARGS: &[&str] = &[
    "repoquery",
    "--userinstalled",
    "--qf",
    "%{name}|%{version}|%{summary}\\n",
];
pub struct Dnf;
impl Provider for Dnf {
    fn id(&self) -> &'static str {
        "dnf"
    }
    fn category(&self) -> Category {
        Category::PackageManager
    }
    fn detect(&self, env: &dyn Env) -> bool {
        matches!(env.os(), Os::Linux(_)) && env.which("dnf")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let raw = run_ok(env, "dnf", ARGS)?;
        let mut items = BTreeMap::new();
        for line in raw.lines().filter(|s| !s.trim().is_empty()) {
            let parts: Vec<_> = line.splitn(3, '|').collect();
            if parts.len() != 3 || parts[0].is_empty() || parts[1].is_empty() {
                return Err(ProviderError::Parse {
                    command: "dnf repoquery --userinstalled".into(),
                    message: format!("invalid package row: {line}"),
                });
            }
            items.entry(parts[0].to_owned()).or_insert_with(|| {
                package_item(
                    "dnf",
                    PackageManager::Dnf,
                    PackageKind::Package,
                    parts[0].into(),
                    Some(parts[1].into()),
                    Some(parts[2].into()),
                    "dnf repoquery --userinstalled",
                )
            });
        }
        Ok(items.into_values().collect())
    }
}
