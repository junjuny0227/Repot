use crate::{catalog, model::*, secrets};
use crate::{env::Env, scan::ProviderError};
use std::path::PathBuf;
pub(crate) fn package_item(
    provider: &str,
    manager: PackageManager,
    kind: PackageKind,
    name: String,
    version: Option<String>,
    description: Option<String>,
    command: &str,
) -> Item {
    Item {
        id: format!("{provider}:{name}"),
        provider: provider.into(),
        category: Category::PackageManager,
        name,
        version,
        description,
        origin: Origin::Command(command.into()),
        portability: Portability::SameOs,
        details: ItemDetails::Package { manager, kind },
        findings: vec![],
    }
}

pub(crate) struct ConfigFileSpec {
    pub provider: &'static str,
    pub category: Category,
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub target: TargetPath,
    pub portability: Portability,
    pub catalog_key: &'static str,
}
pub(crate) fn config_file_item(
    env: &dyn Env,
    spec: ConfigFileSpec,
    extra: impl Fn(&str) -> Vec<Finding>,
) -> Option<Item> {
    if !env.exists(&spec.path) {
        return None;
    }
    let findings = match env.read_file(&spec.path) {
        Ok(content) => {
            let mut f = secrets::findings(&content);
            f.extend(extra(&content));
            f
        }
        Err(_) => vec![Finding {
            line: None,
            kind: FindingKind::Unreadable,
        }],
    };
    Some(Item {
        id: spec.id,
        provider: spec.provider.into(),
        category: spec.category,
        name: spec.name,
        version: None,
        description: catalog::describe(spec.catalog_key).map(str::to_owned),
        origin: Origin::Path(spec.path.clone()),
        portability: spec.portability,
        details: ItemDetails::ConfigFile {
            source: spec.path,
            target: spec.target,
        },
        findings,
    })
}

pub(crate) fn run_ok(env: &dyn Env, program: &str, args: &[&str]) -> Result<String, ProviderError> {
    let out = env.run(program, args)?;
    if !out.success() {
        return Err(ProviderError::CommandFailed {
            command: format!("{} {}", program, args.join(" ")),
            code: out.code,
            stderr: out.stderr,
        });
    }
    Ok(out.stdout)
}

pub(crate) struct RuntimeSpec<'a> {
    pub provider: &'static str,
    pub tool: RuntimeTool,
    pub runtime: &'a str,
    pub name: &'a str,
    pub version: String,
    pub is_default: bool,
    pub origin: Origin,
    pub catalog_key: &'static str,
}
pub(crate) fn runtime_item(spec: RuntimeSpec) -> Item {
    Item {
        id: format!("{}:{}:{}", spec.provider, spec.runtime, spec.version),
        provider: spec.provider.into(),
        category: Category::Runtime,
        name: spec.name.into(),
        version: Some(spec.version),
        description: catalog::describe(spec.catalog_key).map(str::to_owned),
        origin: spec.origin,
        portability: Portability::Mapped(spec.tool),
        details: ItemDetails::Runtime {
            tool: spec.tool,
            runtime: spec.runtime.into(),
            is_default: spec.is_default,
        },
        findings: vec![],
    }
}

pub(crate) fn global_item(
    provider: &str,
    manager: GlobalManager,
    name: String,
    version: Option<String>,
    description: Option<String>,
    command: &str,
) -> Item {
    Item {
        id: format!("{provider}:{name}"),
        provider: provider.into(),
        category: Category::GlobalPackage,
        name,
        version,
        description,
        origin: Origin::Command(command.into()),
        portability: Portability::Any,
        details: ItemDetails::GlobalPackage { manager },
        findings: vec![],
    }
}
