use super::common::{package_item, run_ok};
use crate::{
    env::Env,
    model::*,
    os::Os,
    scan::{Provider, ProviderError},
};
use regex::Regex;
use std::{collections::BTreeSet, path::PathBuf};

fn decode(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

pub struct Choco;
impl Provider for Choco {
    fn id(&self) -> &'static str {
        "choco"
    }
    fn category(&self) -> Category {
        Category::PackageManager
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.os() == Os::Windows && env.which("choco")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let raw = run_ok(env, "choco", &["list", "--limit-output"])?;
        let root = env
            .var("ChocolateyInstall")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:/ProgramData/chocolatey"));
        let summary =
            Regex::new(r"(?s)<summary(?:\s[^>]*)?>(.*?)</summary>").expect("summary regex");
        let description = Regex::new(r"(?s)<description(?:\s[^>]*)?>(.*?)</description>")
            .expect("description regex");
        let dependency = Regex::new(r#"<dependency\b[^>]*?\bid\s*=\s*["']([^"']+)["']"#)
            .expect("dependency regex");
        let mut items = Vec::new();
        let mut deps = BTreeSet::new();
        for line in raw.lines().map(str::trim).filter(|s| !s.is_empty()) {
            let parts: Vec<_> = line.split('|').collect();
            if parts.len() != 2 || parts.iter().any(|s| s.is_empty()) {
                return Err(ProviderError::Parse {
                    command: "choco list --limit-output".into(),
                    message: format!("invalid package row: {line}"),
                });
            }
            let (name, version) = (parts[0], parts[1]);
            if name.eq_ignore_ascii_case("chocolatey") {
                continue;
            }
            let nuspec = env
                .read_file(&root.join("lib").join(name).join(format!("{name}.nuspec")))
                .ok();
            let desc = nuspec.as_deref().and_then(|xml| {
                for capture in dependency.captures_iter(xml) {
                    if !capture[1].eq_ignore_ascii_case(name) {
                        deps.insert(capture[1].to_ascii_lowercase());
                    }
                }
                summary
                    .captures(xml)
                    .map(|c| decode(c[1].trim()))
                    .or_else(|| {
                        description
                            .captures(xml)
                            .and_then(|c| c[1].trim().lines().next().map(|s| decode(s.trim())))
                    })
            });
            items.push(package_item(
                "choco",
                PackageManager::Choco,
                PackageKind::Package,
                name.into(),
                Some(version.into()),
                desc,
                "choco list --limit-output",
            ));
        }
        items.retain(|i| !deps.contains(&i.name.to_ascii_lowercase()));
        Ok(items)
    }
}
