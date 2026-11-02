use super::common::{global_item, run_ok};
use crate::{
    env::Env,
    model::*,
    scan::{Provider, ProviderError},
};
use regex::Regex;

const COMMAND: &str = "cargo install --list";

pub struct Cargo;

impl Provider for Cargo {
    fn id(&self) -> &'static str {
        "cargo"
    }
    fn category(&self) -> Category {
        Category::GlobalPackage
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.which("cargo")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let out = run_ok(env, "cargo", &["install", "--list"])?;
        let re = Regex::new(r"^(\S+) v([^\s:]+)(?: \(.+\))?:$").expect("valid regex");
        let mut items = vec![];
        for line in out.lines().map(|l| l.trim_end_matches('\r')) {
            // Indented lines name the installed binaries of the crate above.
            if line.is_empty() || line.starts_with(char::is_whitespace) {
                continue;
            }
            let c = re.captures(line).ok_or_else(|| ProviderError::Parse {
                command: COMMAND.into(),
                message: format!("unexpected line: {line}"),
            })?;
            items.push(global_item(
                "cargo",
                GlobalManager::Cargo,
                c[1].to_owned(),
                Some(c[2].to_owned()),
                None,
                COMMAND,
            ));
        }
        Ok(items)
    }
}
