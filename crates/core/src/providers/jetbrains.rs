//! JetBrains IDE settings and plugins (plan 14). Only the newest version folder of each IDE
//! is read; restore scripts pick the destination folder when they run.

use super::common::{ConfigFileSpec, config_file_item};
use crate::{
    env::{Env, EnvError},
    jetbrains::version_key,
    model::*,
    os::Os,
    scan::{Provider, ProviderError},
};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct JetBrains;

/// Folders whose files carry over between machines, with their catalog entries.
const SETTINGS_DIRS: [(&str, &str, &str); 4] = [
    ("codestyles", ".xml", "file:jetbrains-codestyle"),
    ("keymaps", ".xml", "file:jetbrains-keymap"),
    ("colors", ".icls", "file:jetbrains-colors"),
    ("templates", ".xml", "file:jetbrains-templates"),
];
/// Font and theme choices. The rest of `options/` holds paths, JDKs, recent projects and
/// accounts of this machine and is left out.
const OPTION_FILES: [&str; 4] = [
    "editor-font.xml",
    "console-font.xml",
    "colors.scheme.xml",
    "ui.lnf.xml",
];

fn config_base(env: &dyn Env) -> PathBuf {
    let home = env.home_dir();
    match env.os() {
        Os::MacOs => home.join("Library/Application Support/JetBrains"),
        Os::Windows => env
            .var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join("AppData/Roaming"))
            .join("JetBrains"),
        Os::Linux(_) => env
            .var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"))
            .join("JetBrains"),
    }
}

fn plugins_dir(env: &dyn Env, config: &Path, folder: &str) -> PathBuf {
    match env.os() {
        Os::Linux(_) => env
            .var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| env.home_dir().join(".local/share"))
            .join("JetBrains")
            .join(folder),
        _ => config.join("plugins"),
    }
}

/// The newest version folder of each IDE.
fn newest_folders(env: &dyn Env) -> Result<BTreeMap<JetBrainsIde, String>, ProviderError> {
    let entries = match env.list_dir(&config_base(env)) {
        Ok(entries) => entries,
        Err(EnvError::NotFound) => return Ok(BTreeMap::new()),
        Err(e) => return Err(e.into()),
    };
    let mut newest: BTreeMap<JetBrainsIde, String> = BTreeMap::new();
    for path in entries {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some((ide, version)) = JetBrainsIde::parse_folder(name) else {
            continue;
        };
        if newest
            .get(&ide)
            .is_none_or(|old| version_key(&version) > version_key(old))
        {
            newest.insert(ide, version);
        }
    }
    Ok(newest)
}

/// Fields read from a plugin's `META-INF/plugin.xml`.
#[derive(Debug, PartialEq)]
struct PluginInfo {
    id: String,
    name: String,
    version: Option<String>,
    description: Option<String>,
}

fn tag<'a>(xml: &'a str, name: &str) -> Option<&'a str> {
    let open = format!("<{name}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&format!("</{name}>"))? + start;
    Some(xml[start..end].trim())
}

/// First sentence of an HTML description, tags and CDATA removed.
fn first_sentence(html: &str) -> Option<String> {
    let html = html
        .trim()
        .trim_start_matches("<![CDATA[")
        .trim_end_matches("]]>");
    // Without CDATA the HTML arrives escaped (`&lt;p&gt;`).
    let html = html.replace("&lt;", "<").replace("&gt;", ">");
    let mut text = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                text.push(' ');
            }
            '>' => in_tag = false,
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    let text = text
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&");
    let text = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        // A closing tag before punctuation left a space: "Spring Framework ."
        .replace(" .", ".")
        .replace(" ,", ",");
    let sentence = match text.find(". ") {
        Some(end) => &text[..=end],
        None => text.as_str(),
    };
    let sentence: String = sentence.chars().take(200).collect();
    (!sentence.is_empty()).then_some(sentence)
}

fn plugin_info(xml: &str) -> Option<PluginInfo> {
    let name = tag(xml, "name")?.to_owned();
    // Without <id>, the IDE uses <name> as the id.
    let id = tag(xml, "id").map_or_else(|| name.clone(), str::to_owned);
    if id.is_empty() || name.is_empty() {
        return None;
    }
    Some(PluginInfo {
        id,
        name,
        version: tag(xml, "version").map(str::to_owned),
        description: tag(xml, "description").and_then(first_sentence),
    })
}

fn read_jar_plugin(env: &dyn Env, jar: &Path) -> Option<PluginInfo> {
    let bytes = env.read_bytes(jar).ok()?;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).ok()?;
    let mut entry = archive.by_name("META-INF/plugin.xml").ok()?;
    let mut xml = String::new();
    entry.read_to_string(&mut xml).ok()?;
    plugin_info(&xml)
}

/// A plugin is a folder with `lib/*.jar` (the jar named after the folder usually holds
/// plugin.xml) or a single jar.
fn read_plugin(env: &dyn Env, path: &Path) -> Option<PluginInfo> {
    let name = path.file_name()?.to_str()?;
    if name.ends_with(".jar") {
        return read_jar_plugin(env, path);
    }
    let mut jars: Vec<PathBuf> = env
        .list_dir(&path.join("lib"))
        .ok()?
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "jar"))
        .collect();
    jars.sort_by_key(|jar| {
        let file = jar.file_name().and_then(|n| n.to_str()).unwrap_or("");
        (!file.starts_with(name), file.to_owned())
    });
    jars.iter().find_map(|jar| read_jar_plugin(env, jar))
}

impl Provider for JetBrains {
    fn id(&self) -> &'static str {
        "jetbrains"
    }
    fn category(&self) -> Category {
        Category::Editor
    }
    fn detect(&self, env: &dyn Env) -> bool {
        newest_folders(env).is_ok_and(|folders| !folders.is_empty())
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let base = config_base(env);
        let mut items = vec![];
        for (ide, version) in newest_folders(env)? {
            let folder = format!("{}{version}", ide.prefix());
            let config = base.join(&folder);
            let plugins = plugins_dir(env, &config, &folder);
            match env.list_dir(&plugins) {
                Ok(paths) => {
                    for path in paths {
                        let Some(info) = read_plugin(env, &path) else {
                            continue;
                        };
                        let folder_name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        items.push(Item {
                            id: format!("jetbrains:{}:plugin:{}", ide.prefix(), info.id),
                            provider: "jetbrains".into(),
                            category: self.category(),
                            name: info.id,
                            version: info.version,
                            description: info.description.or(Some(info.name)),
                            catalog_id: None,
                            origin: Origin::Path(path),
                            portability: Portability::Any,
                            details: ItemDetails::IdePlugin {
                                ide,
                                folder: folder_name,
                            },
                            findings: vec![],
                        });
                    }
                }
                Err(EnvError::NotFound) => {}
                Err(e) => return Err(e.into()),
            }
            let mut files: Vec<(PathBuf, &'static str)> = vec![];
            for (dir, ext, catalog_key) in SETTINGS_DIRS {
                match env.list_dir(&config.join(dir)) {
                    Ok(paths) => files.extend(
                        paths
                            .into_iter()
                            .filter_map(|p| {
                                let file = p.file_name()?.to_str()?.to_owned();
                                file.ends_with(ext).then(|| PathBuf::from(dir).join(file))
                            })
                            .map(|rel| (rel, catalog_key)),
                    ),
                    Err(EnvError::NotFound) => {}
                    Err(e) => return Err(e.into()),
                }
            }
            files.extend(
                OPTION_FILES
                    .iter()
                    .map(|f| (PathBuf::from("options").join(f), "file:jetbrains-options")),
            );
            for (rel, catalog_key) in files {
                let rel_text = rel.to_string_lossy().replace('\\', "/");
                if let Some(item) = config_file_item(
                    env,
                    ConfigFileSpec {
                        provider: "jetbrains",
                        category: self.category(),
                        id: format!("jetbrains:{}:file:{rel_text}", ide.prefix()),
                        name: rel_text,
                        path: config.join(&rel),
                        target: TargetPath::JetBrains {
                            ide,
                            version: version.clone(),
                            rel,
                        },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_plugin_xml_fields() {
        let xml = "<idea-plugin><id>com.github.copilot</id><name>GitHub Copilot</name>\
                   <version>1.7.1</version><depends>com.intellij.modules.platform</depends>\
                   <description><![CDATA[<p>AI pair programmer. Works in <b>every</b> IDE.</p>]]></description></idea-plugin>";
        assert_eq!(
            plugin_info(xml),
            Some(PluginInfo {
                id: "com.github.copilot".into(),
                name: "GitHub Copilot".into(),
                version: Some("1.7.1".into()),
                description: Some("AI pair programmer.".into()),
            })
        );
        let escaped = "<idea-plugin><id>a</id><name>A</name><description>&lt;p&gt;&lt;b&gt;Gone \
                       soon.&lt;/b&gt; Still here.&lt;/p&gt;</description></idea-plugin>";
        assert_eq!(
            plugin_info(escaped).and_then(|p| p.description),
            Some("Gone soon.".into())
        );
        assert_eq!(
            first_sentence("<p>Inspections for <a href=\"x\">Spring</a>.</p>"),
            Some("Inspections for Spring.".into())
        );
        // No <id>: the IDE uses the name.
        assert_eq!(
            plugin_info("<idea-plugin><name>Ideolog</name></idea-plugin>").map(|p| p.id),
            Some("Ideolog".into())
        );
        assert_eq!(plugin_info("<idea-plugin></idea-plugin>"), None);
    }
}
