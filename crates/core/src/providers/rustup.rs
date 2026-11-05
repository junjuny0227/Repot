use super::common::{RuntimeSpec, run_ok, runtime_item};
use crate::{
    env::Env,
    model::*,
    scan::{Provider, ProviderError},
};
use regex::Regex;
use std::collections::BTreeSet;

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
        // Channels in output order; the same channel for two host triples becomes one item
        // with the components and targets of both.
        let mut channels: Vec<Channel> = vec![];
        for line in out.lines() {
            let Some(first) = line.split_whitespace().next() else {
                continue;
            };
            // Custom linked toolchains do not match and are skipped.
            let Some(c) = re.captures(first) else {
                continue;
            };
            let is_default = line.contains("default");
            let index = match channels.iter().position(|ch| ch.version == c[1]) {
                Some(index) => index,
                None => {
                    channels.push(Channel {
                        version: c[1].to_owned(),
                        ..Channel::default()
                    });
                    channels.len() - 1
                }
            };
            let channel = &mut channels[index];
            channel.is_default |= is_default;
            let host = first
                .strip_prefix(&c[1])
                .unwrap_or("")
                .trim_start_matches('-');
            channel.components.extend(components(env, first, host));
            channel.targets.extend(targets(env, first, host));
        }
        Ok(channels
            .into_iter()
            .map(|channel| {
                let Channel {
                    version,
                    is_default,
                    components,
                    targets,
                } = channel;
                let mut item = runtime_item(RuntimeSpec {
                    provider: "rustup",
                    tool: RuntimeTool::Rustup,
                    runtime: "rust",
                    name: "Rust toolchain",
                    version,
                    is_default,
                    origin: Origin::Command("rustup toolchain list".into()),
                    catalog_key: "runtime:rust",
                });
                if let ItemDetails::Runtime {
                    components: c,
                    targets: t,
                    ..
                } = &mut item.details
                {
                    *c = components.into_iter().collect();
                    *t = targets.into_iter().collect();
                }
                item
            })
            .collect())
    }
}

#[derive(Default)]
struct Channel {
    version: String,
    is_default: bool,
    components: BTreeSet<String>,
    targets: BTreeSet<String>,
}

/// A component or target name as rustup prints it: lowercase ASCII letters, digits, `.`,
/// `_` and `-`, not starting with punctuation (so never an option), at most 64 bytes.
/// Restore scripts and opened Repot files accept only these.
pub fn is_rustup_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    (1..=64).contains(&bytes.len())
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(b))
}

/// Installed components of `toolchain` without the `-<host>` suffix rustup adds to
/// host-specific ones. `rust-std-*` entries are skipped: they are the installed targets.
/// A failing command keeps the toolchain without components, as before they were scanned.
fn components(env: &dyn Env, toolchain: &str, host: &str) -> Vec<String> {
    let suffix = format!("-{host}");
    list(env, "component", toolchain)
        .into_iter()
        .map(|name| match name.strip_suffix(&suffix) {
            Some(short) if !host.is_empty() => short.to_owned(),
            _ => name,
        })
        .filter(|name| name != "rust-std" && !name.starts_with("rust-std-"))
        .collect()
}

/// Installed targets of `toolchain` except its host, which every toolchain has.
fn targets(env: &dyn Env, toolchain: &str, host: &str) -> Vec<String> {
    list(env, "target", toolchain)
        .into_iter()
        .filter(|name| name != host)
        .collect()
}

fn list(env: &dyn Env, kind: &str, toolchain: &str) -> Vec<String> {
    let Ok(out) = run_ok(
        env,
        "rustup",
        &[kind, "list", "--installed", "--toolchain", toolchain],
    ) else {
        return vec![];
    };
    out.lines()
        .map(str::trim)
        .filter(|name| is_rustup_name(name))
        .map(str::to_owned)
        .collect()
}
