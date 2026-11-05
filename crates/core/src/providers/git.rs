use super::common::{ConfigFileSpec, config_file_item};
use crate::{
    env::Env,
    model::*,
    scan::{Provider, ProviderError},
};
pub struct Git;
/// Config files read for every user, in reading order.
const CONFIG_FILES: [&str; 2] = [".gitconfig", ".config/git/config"];
/// The global ignore file when `core.excludesFile` is not set.
const DEFAULT_IGNORE: &str = ".config/git/ignore";

/// Whether an opened Repot file may restore a git item to `rel` (home-relative, `/`).
/// A custom `core.excludesFile` is accepted only as a file named like an ignore file,
/// directly in home or in `.config/git/`: anything else could be an SSH key list, a
/// login item or an autostart entry that Repot itself never writes.
pub(crate) fn is_config_target(rel: &str) -> bool {
    if CONFIG_FILES.contains(&rel) || rel == DEFAULT_IGNORE {
        return true;
    }
    let (dir, file) = rel.rsplit_once('/').unwrap_or(("", rel));
    (dir.is_empty() || dir == ".config/git") && file.to_ascii_lowercase().contains("ignore")
}
// Preserve source line numbers, including CRLF, and normalize only section/key names.
pub(crate) fn entries(content: &str) -> Vec<(u32, String, String, String)> {
    let mut section = String::new();
    let mut result = vec![];
    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.starts_with('#') || line.starts_with(';') || line.is_empty() {
            continue;
        }
        if let Some(s) = line.strip_prefix('[').and_then(|s| s.split(']').next()) {
            section = s.trim().to_ascii_lowercase();
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            result.push((
                (i + 1) as u32,
                section.clone(),
                key.trim().to_ascii_lowercase(),
                value.trim().trim_matches('"').to_owned(),
            ));
        }
    }
    result
}
pub(crate) fn os_specific_lines(content: &str) -> Vec<Finding> {
    entries(content)
        .into_iter()
        .filter_map(|(line, section, key, value)| {
            let section = section.split_whitespace().next().unwrap_or("");
            let key = format!("{section}.{key}");
            (matches!(
                key.as_str(),
                "credential.helper" | "core.autocrlf" | "gpg.program"
            ) || (key == "core.editor" && (value.contains('/') || value.contains('\\'))))
            .then_some(Finding {
                line: Some(line),
                kind: FindingKind::OsSpecificLine { key },
            })
        })
        .collect()
}
impl Provider for Git {
    fn id(&self) -> &'static str {
        "git"
    }
    fn category(&self) -> Category {
        Category::Git
    }
    fn detect(&self, env: &dyn Env) -> bool {
        CONFIG_FILES
            .iter()
            .chain([&DEFAULT_IGNORE])
            .any(|p| env.exists(&env.home_dir().join(p)))
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let home = env.home_dir();
        let mut items = vec![];
        let mut ignore = None;
        for rel in CONFIG_FILES {
            let path = home.join(rel);
            if let Ok(content) = env.read_file(&path) {
                for (_, section, key, value) in entries(&content) {
                    if section == "core" && key == "excludesfile" {
                        ignore = Some(if let Some(rel) = value.strip_prefix("~/") {
                            home.join(rel)
                        } else {
                            let p = std::path::PathBuf::from(value);
                            if p.is_absolute() { p } else { home.join(p) }
                        });
                    }
                }
            }
            if let Some(item) = config_file_item(
                env,
                ConfigFileSpec {
                    provider: "git",
                    category: self.category(),
                    id: format!("git:file:{rel}"),
                    name: rel.into(),
                    path,
                    target: TargetPath::Home(rel.into()),
                    portability: Portability::Any,
                    catalog_key: "file:gitconfig",
                },
                os_specific_lines,
            ) {
                items.push(item);
            }
        }
        let path = ignore.unwrap_or_else(|| home.join(DEFAULT_IGNORE));
        let rel = path
            .strip_prefix(&home)
            .unwrap_or(std::path::Path::new(DEFAULT_IGNORE))
            .to_path_buf();
        if !items.iter().any(|i| i.origin == Origin::Path(path.clone()))
            && let Some(item) = config_file_item(
                env,
                ConfigFileSpec {
                    provider: "git",
                    category: self.category(),
                    id: format!("git:file:{}", rel.to_string_lossy()),
                    name: rel.to_string_lossy().into_owned(),
                    path,
                    target: TargetPath::Home(rel),
                    portability: Portability::Any,
                    catalog_key: "file:gitignore-global",
                },
                |_| vec![],
            )
        {
            items.push(item);
        }
        Ok(items)
    }
}
