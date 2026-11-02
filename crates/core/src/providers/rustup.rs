use super::common::{RuntimeSpec, run_ok, runtime_item};
use crate::{
    env::Env,
    model::*,
    scan::{Provider, ProviderError},
};
use regex::Regex;

pub struct Rustup;

impl Provider for Rustup {
    fn id(&self) -> &'static str {
        "rustup"
    }
    fn category(&self) -> Category {
        Category::Runtime
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.which("rustup")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let out = run_ok(env, "rustup", &["toolchain", "list"])?;
        let re = Regex::new(
            r"^(stable|beta|nightly(?:-\d{4}-\d{2}-\d{2})?|\d+\.\d+(?:\.\d+)?)(?:-.+)?$",
        )
        .expect("valid regex");
        // Channels in output order; the same channel for two host triples becomes one item.
        let mut channels: Vec<(String, bool)> = vec![];
        for line in out.lines() {
            let Some(first) = line.split_whitespace().next() else {
                continue;
            };
            // Custom linked toolchains do not match and are skipped.
            let Some(c) = re.captures(first) else {
                continue;
            };
            let is_default = line.contains("default");
            match channels.iter_mut().find(|(ch, _)| ch == &c[1]) {
                Some(existing) => existing.1 |= is_default,
                None => channels.push((c[1].to_owned(), is_default)),
            }
        }
        Ok(channels
            .into_iter()
            .map(|(version, is_default)| {
                runtime_item(RuntimeSpec {
                    provider: "rustup",
                    tool: RuntimeTool::Rustup,
                    runtime: "rust",
                    name: "Rust toolchain",
                    version,
                    is_default,
                    origin: Origin::Command("rustup toolchain list".into()),
                    catalog_key: "runtime:rust",
                })
            })
            .collect())
    }
}
