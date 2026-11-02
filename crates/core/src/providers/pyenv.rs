use super::common::{RuntimeSpec, run_ok, runtime_item};
use crate::{
    env::Env,
    model::*,
    os::Os,
    scan::{Provider, ProviderError},
};

pub struct Pyenv;

impl Provider for Pyenv {
    fn id(&self) -> &'static str {
        "pyenv"
    }
    fn category(&self) -> Category {
        Category::Runtime
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.which("pyenv")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let out = run_ok(env, "pyenv", &["versions", "--bare"])?;
        // `pyenv global` without a configured version is not a scan failure: no default.
        let global = env
            .run("pyenv", &["global"])
            .ok()
            .filter(|o| o.success())
            .and_then(|o| o.stdout.lines().next().map(|l| l.trim().to_owned()))
            .filter(|v| !v.is_empty() && v != "system");
        let tool = if env.os() == Os::Windows {
            RuntimeTool::PyenvWin
        } else {
            RuntimeTool::Pyenv
        };
        let mut seen = std::collections::BTreeSet::new();
        Ok(out
            .lines()
            .map(str::trim)
            .filter(|v| !v.is_empty() && *v != "system")
            .filter(|v| seen.insert(v.to_string()))
            .map(|version| {
                runtime_item(RuntimeSpec {
                    provider: "pyenv",
                    tool,
                    runtime: "python",
                    name: "Python",
                    version: version.to_owned(),
                    is_default: global.as_deref() == Some(version),
                    origin: Origin::Command("pyenv versions --bare".into()),
                    catalog_key: "runtime:python",
                })
            })
            .collect())
    }
}
