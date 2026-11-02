use super::common::{package_item, run_ok};
use crate::{
    env::Env,
    model::*,
    os::Os,
    scan::{Provider, ProviderError},
};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Export {
    buckets: Vec<Bucket>,
    apps: Vec<App>,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Bucket {
    name: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct App {
    name: String,
    source: String,
    version: Option<String>,
}

pub struct Scoop;
impl Provider for Scoop {
    fn id(&self) -> &'static str {
        "scoop"
    }
    fn category(&self) -> Category {
        Category::PackageManager
    }
    fn detect(&self, env: &dyn Env) -> bool {
        env.os() == Os::Windows && env.which("scoop")
    }
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        let raw = run_ok(env, "scoop", &["export"])?;
        let export: Export = serde_json::from_str(&raw).map_err(|e| ProviderError::Parse {
            command: "scoop export".into(),
            message: e.to_string(),
        })?;
        let root = env
            .var("SCOOP")
            .map(PathBuf::from)
            .unwrap_or_else(|| env.home_dir().join("scoop"));
        let mut items = Vec::new();
        for bucket in export.buckets.into_iter().filter(|b| b.name != "main") {
            let mut item = package_item(
                "scoop",
                PackageManager::Scoop,
                PackageKind::Tap,
                bucket.name,
                None,
                None,
                "scoop export",
            );
            item.id = format!("scoop:bucket:{}", item.name);
            items.push(item);
        }
        for app in export.apps {
            let description = env
                .read_file(
                    &root
                        .join("buckets")
                        .join(&app.source)
                        .join("bucket")
                        .join(format!("{}.json", app.name)),
                )
                .ok()
                .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
                .and_then(|v| v.get("description")?.as_str().map(str::to_owned));
            items.push(package_item(
                "scoop",
                PackageManager::Scoop,
                PackageKind::Package,
                format!("{}/{}", app.source, app.name),
                app.version,
                description,
                "scoop export",
            ));
        }
        Ok(items)
    }
}
