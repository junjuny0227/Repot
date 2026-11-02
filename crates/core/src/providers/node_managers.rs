use super::common::{RuntimeSpec, run_ok, runtime_item};
use crate::{
    env::Env,
    model::*,
    os::Os,
    scan::{Provider, ProviderError},
};
use regex::Regex;

pub struct NvmWindows;
pub struct Fnm;
pub struct Volta;

fn numeric(s: &str) -> Vec<u64> {
    s.split('.').filter_map(|v| v.parse().ok()).collect()
}

/// Builds sorted Node.js runtime items from `(version, is_default)` pairs.
fn node_items(
    provider: &'static str,
    tool: RuntimeTool,
    command: &str,
    mut versions: Vec<(String, bool)>,
) -> Vec<Item> {
    versions.sort_by_key(|(v, _)| numeric(v));
    versions.dedup_by(|a, b| {
        let same = a.0 == b.0;
        if same {
            b.1 |= a.1;
        }
        same
    });
    versions
        .into_iter()
        .map(|(version, is_default)| {
            runtime_item(RuntimeSpec {
                provider,
                tool,
                runtime: "node",
                name: "Node.js",
                version,
                is_default,
                origin: Origin::Command(command.into()),
                catalog_key: "runtime:node",
            })
        })
        .collect()
}

fn parse_lines(out: &str, re: &Regex, is_default: impl Fn(&str) -> bool) -> Vec<(String, bool)> {
    out.lines()
        .filter_map(|line| {
            re.captures(line)
                .map(|c| (c[1].to_owned(), is_default(line)))
        })
        .collect()
}

impl Provider for NvmWindows {
    fn id(&self) -> &'static str {
        "nvm-windows"
    }
    fn category(&self) -> Category {
        Category::Runtime
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.os() == Os::Windows && env.which("nvm")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let out = run_ok(env, "nvm", &["list"])?;
        let re = Regex::new(r"(\d+\.\d+\.\d+)").expect("valid regex");
        let versions = parse_lines(&out, &re, |l| l.contains('*'));
        Ok(node_items(
            self.id(),
            RuntimeTool::NvmWindows,
            "nvm list",
            versions,
        ))
    }
}

impl Provider for Fnm {
    fn id(&self) -> &'static str {
        "fnm"
    }
    fn category(&self) -> Category {
        Category::Runtime
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.which("fnm")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let out = run_ok(env, "fnm", &["list"])?;
        let re = Regex::new(r"\bv(\d+\.\d+\.\d+)\b").expect("valid regex");
        let versions = parse_lines(&out, &re, |l| {
            l.split_whitespace()
                .any(|w| w.trim_matches(',') == "default")
        });
        Ok(node_items(
            self.id(),
            RuntimeTool::Fnm,
            "fnm list",
            versions,
        ))
    }
}

impl Provider for Volta {
    fn id(&self) -> &'static str {
        "volta"
    }
    fn category(&self) -> Category {
        Category::Runtime
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.which("volta")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let out = run_ok(env, "volta", &["list", "--format", "plain"])?;
        let re = Regex::new(r"^runtime node@(\d+\.\d+\.\d+)\b").expect("valid regex");
        let versions = parse_lines(&out, &re, |l| l.contains("(default)"));
        Ok(node_items(
            self.id(),
            RuntimeTool::Volta,
            "volta list --format plain",
            versions,
        ))
    }
}
