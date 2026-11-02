use super::common::{package_item, run_ok};
use crate::{
    env::Env,
    model::*,
    os::Os,
    scan::{Provider, ProviderError},
};
use std::path::Path;

pub struct Pacman;
impl Provider for Pacman {
    fn id(&self) -> &'static str {
        "pacman"
    }
    fn category(&self) -> Category {
        Category::PackageManager
    }
    fn detect(&self, env: &dyn Env) -> bool {
        matches!(env.os(), Os::Linux(_)) && env.which("pacman")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let raw = run_ok(env, "pacman", &["-Qe"])?;
        raw.lines()
            .filter(|s| !s.trim().is_empty())
            .map(|line| {
                let parts: Vec<_> = line.split_whitespace().collect();
                let [name, version] = parts.as_slice() else {
                    return Err(ProviderError::Parse {
                        command: "pacman -Qe".into(),
                        message: format!("invalid package row: {line}"),
                    });
                };
                let desc = env
                    .read_file(
                        &Path::new("/var/lib/pacman/local")
                            .join(format!("{name}-{version}"))
                            .join("desc"),
                    )
                    .ok()
                    .and_then(|content| {
                        let mut lines = content.lines();
                        lines.find(|line| line.trim() == "%DESC%")?;
                        lines
                            .next()
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(str::to_owned)
                    });
                Ok(package_item(
                    "pacman",
                    PackageManager::Pacman,
                    PackageKind::Package,
                    (*name).into(),
                    Some((*version).into()),
                    desc,
                    "pacman -Qe",
                ))
            })
            .collect()
    }
}
