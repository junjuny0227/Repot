use super::common::{package_item, run_ok};
use crate::{
    env::Env,
    model::*,
    os::Os,
    scan::{Provider, ProviderError},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

const DPKG_ARGS: &[&str] = &["-W", "-f=${Package}\\t${Version}\\t${binary:Summary}\\n"];
const INITIAL: &str = "/var/log/installer/initial-status.gz";
fn key(name: &str) -> &str {
    name.split(':').next().unwrap_or(name)
}
fn parse(message: String) -> ProviderError {
    ProviderError::Parse {
        command: "dpkg-query -W".into(),
        message,
    }
}

pub struct Apt;
impl Provider for Apt {
    fn id(&self) -> &'static str {
        "apt"
    }
    fn category(&self) -> Category {
        Category::PackageManager
    }
    fn detect(&self, env: &dyn Env) -> bool {
        matches!(env.os(), Os::Linux(_)) && env.which("apt-mark")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let manual = run_ok(env, "apt-mark", &["showmanual"])?;
        let raw = run_ok(env, "dpkg-query", DPKG_ARGS)?;
        let mut metadata = BTreeMap::new();
        for line in raw.lines().filter(|s| !s.trim().is_empty()) {
            let parts: Vec<_> = line.splitn(3, '\t').collect();
            if parts.len() != 3 || parts[0].is_empty() || parts[1].is_empty() {
                return Err(parse(format!("invalid package row: {line}")));
            }
            metadata.insert(key(parts[0]), (parts[1], parts[2]));
        }
        let initial: BTreeSet<String> = if env.exists(Path::new(INITIAL)) {
            run_ok(env, "gzip", &["-dc", INITIAL])
                .ok()
                .map(|raw| {
                    raw.lines()
                        .filter_map(|line| line.strip_prefix("Package: "))
                        .map(|name| name.trim().to_owned())
                        .collect()
                })
                .unwrap_or_default()
        } else {
            BTreeSet::new()
        };
        let mut names = BTreeSet::new();
        for name in manual.lines().map(str::trim).filter(|s| !s.is_empty()) {
            if name.split_whitespace().count() != 1 {
                return Err(ProviderError::Parse {
                    command: "apt-mark showmanual".into(),
                    message: format!("invalid package name: {name}"),
                });
            }
            if !initial.contains(key(name)) {
                names.insert(name);
            }
        }
        names
            .into_iter()
            .map(|name| {
                let (version, summary) = metadata
                    .get(key(name))
                    .ok_or_else(|| parse(format!("missing package metadata: {name}")))?;
                Ok(package_item(
                    "apt",
                    PackageManager::Apt,
                    PackageKind::Package,
                    name.into(),
                    Some((*version).into()),
                    Some((*summary).into()),
                    "apt-mark showmanual",
                ))
            })
            .collect()
    }
}
