use super::common::package_item;
use crate::{
    env::Env,
    model::*,
    os::Os,
    scan::{Provider, ProviderError},
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Export {
    sources: Vec<Source>,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Source {
    source_details: SourceDetails,
    packages: Vec<Package>,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SourceDetails {
    name: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Package {
    package_identifier: String,
    version: Option<String>,
}

pub struct Winget;
impl Provider for Winget {
    fn id(&self) -> &'static str {
        "winget"
    }
    fn category(&self) -> Category {
        Category::PackageManager
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.os() == Os::Windows && env.which("winget")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let path = env.temp_file("winget-export.json");
        let result = (|| {
            let out = env.run(
                "winget",
                &[
                    "export",
                    "-o",
                    &path.to_string_lossy(),
                    "--include-versions",
                    "--accept-source-agreements",
                    "--disable-interactivity",
                ],
            )?;
            let raw = match env.read_file(&path) {
                Ok(raw) => raw,
                Err(_) if !out.success() => {
                    return Err(ProviderError::CommandFailed {
                        command: "winget export".into(),
                        code: out.code,
                        stderr: out.stderr,
                    });
                }
                Err(e) => return Err(e.into()),
            };
            let export: Export = serde_json::from_str(&raw).map_err(|e| ProviderError::Parse {
                command: "winget export".into(),
                message: e.to_string(),
            })?;
            Ok(export
                .sources
                .into_iter()
                .filter(|s| s.source_details.name == "winget")
                .flat_map(|s| s.packages)
                .map(|p| {
                    package_item(
                        "winget",
                        PackageManager::Winget,
                        PackageKind::Package,
                        p.package_identifier,
                        p.version,
                        None,
                        "winget export",
                    )
                })
                .collect())
        })();
        env.remove_file(&path);
        result
    }
}
