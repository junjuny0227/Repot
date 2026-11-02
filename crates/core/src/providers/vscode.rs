use super::common::{ConfigFileSpec, config_file_item};
use crate::{
    env::{Env, EnvError},
    model::*,
    os::Os,
    scan::{Provider, ProviderError},
};
use serde::Deserialize;
use std::path::{Path, PathBuf};
#[derive(Deserialize)]
struct Extension {
    identifier: Identifier,
    version: String,
    #[serde(rename = "relativeLocation")]
    relative_location: PathBuf,
}
#[derive(Deserialize)]
struct Identifier {
    id: String,
}
pub struct VsCode;
fn roots(env: &dyn Env, editor: Editor) -> (PathBuf, PathBuf) {
    let home = env.home_dir();
    let (dot, app) = if editor == Editor::VsCode {
        (".vscode", "Code")
    } else {
        (".cursor", "Cursor")
    };
    let base = match env.os() {
        Os::MacOs => home.join("Library/Application Support"),
        Os::Windows => env
            .var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join("AppData/Roaming")),
        Os::Linux(_) => home.join(".config"),
    };
    (
        home.join(dot).join("extensions"),
        base.join(app).join("User"),
    )
}
fn description(env: &dyn Env, dir: &Path) -> Option<String> {
    let raw = env.read_file(&dir.join("package.json")).ok()?;
    let json: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let desc = json["description"].as_str()?;
    if let Some(key) = desc.strip_prefix('%').and_then(|s| s.strip_suffix('%')) {
        let raw = env.read_file(&dir.join("package.nls.json")).ok()?;
        let nls: serde_json::Value = serde_json::from_str(&raw).ok()?;
        nls[key].as_str().map(str::to_owned)
    } else {
        Some(desc.to_owned())
    }
}
/// Compares dotted versions numerically (`0.174.0` > `0.99.0`).
fn newer(a: &str, b: &str) -> bool {
    let parts = |v: &str| -> Vec<u64> {
        v.split('.')
            .map(|p| {
                let digits: String = p.chars().take_while(char::is_ascii_digit).collect();
                digits.parse().unwrap_or(0)
            })
            .collect()
    };
    parts(a) > parts(b)
}
impl Provider for VsCode {
    fn id(&self) -> &'static str {
        "vscode"
    }
    fn category(&self) -> Category {
        Category::Editor
    }
    fn detect(&self, env: &dyn Env) -> bool {
        [Editor::VsCode, Editor::Cursor].into_iter().any(|e| {
            let (ext, user) = roots(env, e);
            env.exists(&ext) || env.exists(&user)
        })
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let mut items = vec![];
        for (editor, label) in [(Editor::VsCode, "code"), (Editor::Cursor, "cursor")] {
            let (ext, user) = roots(env, editor);
            let path = ext.join("extensions.json");
            match env.read_file(&path) {
                Ok(raw) => {
                    let extensions: Vec<Extension> =
                        serde_json::from_str(&raw).map_err(|e| ProviderError::Parse {
                            command: path.to_string_lossy().into_owned(),
                            message: e.to_string(),
                        })?;
                    for e in extensions {
                        // A registry location is relative to its extension root, never an arbitrary local path.
                        if e.relative_location.is_absolute()
                            || e.relative_location
                                .components()
                                .any(|c| matches!(c, std::path::Component::ParentDir))
                        {
                            return Err(ProviderError::Parse {
                                command: path.to_string_lossy().into_owned(),
                                message: "invalid relativeLocation".into(),
                            });
                        }
                        let name = e.identifier.id;
                        let id = format!("vscode:{label}:ext:{name}");
                        // After an update VS Code keeps the old entry until it cleans up; keep the newest.
                        let existing = items.iter().position(|i: &Item| i.id == id);
                        if let Some(at) = existing
                            && !newer(&e.version, items[at].version.as_deref().unwrap_or(""))
                        {
                            continue;
                        }
                        let item = Item {
                            id,
                            provider: "vscode".into(),
                            category: self.category(),
                            name,
                            version: Some(e.version),
                            description: description(env, &ext.join(e.relative_location)),
                            origin: Origin::Path(path.clone()),
                            portability: Portability::Any,
                            details: ItemDetails::Extension { editor },
                            findings: vec![],
                        };
                        match existing {
                            Some(at) => items[at] = item,
                            None => items.push(item),
                        }
                    }
                }
                Err(EnvError::NotFound) => {}
                Err(e) => return Err(e.into()),
            }
            let mut files = vec![
                (PathBuf::from("settings.json"), "file:vscode-settings"),
                (PathBuf::from("keybindings.json"), "file:vscode-keybindings"),
            ];
            match env.list_dir(&user.join("snippets")) {
                Ok(mut paths) => {
                    paths.sort();
                    for path in paths {
                        files.push((
                            PathBuf::from("snippets").join(path.file_name().unwrap()),
                            "file:vscode-snippet",
                        ));
                    }
                }
                Err(EnvError::NotFound) => {}
                Err(e) => return Err(e.into()),
            }
            for (rel, catalog_key) in files {
                if let Some(item) = config_file_item(
                    env,
                    ConfigFileSpec {
                        provider: "vscode",
                        category: self.category(),
                        id: format!(
                            "vscode:{label}:file:{}",
                            rel.to_string_lossy().replace('\\', "/")
                        ),
                        name: rel.to_string_lossy().into_owned(),
                        path: user.join(&rel),
                        target: TargetPath::EditorUser { editor, rel },
                        portability: Portability::Any,
                        catalog_key,
                    },
                    |_| vec![],
                ) {
                    items.push(item);
                }
            }
        }
        Ok(items)
    }
}
