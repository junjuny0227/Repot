//! Export real restore inputs for disposable CI runners; never execute them here.
use repot_core::{
    FakeEnv,
    export::{ExportBundle, ExportMeta, ExportRequest, Format, build_export, write_export},
    model::*,
    os::{LinuxFamily, Os},
};
use std::{error::Error, path::PathBuf};

const MISSING: &str = "repot-e2e-nonexistent-package-7f49c2";
const CONTENT: &str = "repot restore e2e\n";

fn e2e_bundle(os: Os) -> ExportBundle {
    let (manager, kind, package) = match os {
        Os::MacOs => (PackageManager::Brew, PackageKind::Formula, "sl"),
        Os::Windows => (PackageManager::Choco, PackageKind::Package, "dos2unix"),
        Os::Linux(LinuxFamily::Debian) => (PackageManager::Apt, PackageKind::Package, "sl"),
        _ => unreachable!("only the three CI targets are supported"),
    };
    let source = PathBuf::from("/repot-fixture/.repot-e2e");
    let env = FakeEnv::new(os, "/repot-fixture").with_file(&source, CONTENT);
    let item = |name: &str, category, details, portability| Item {
        id: format!("e2e:{name}"),
        provider: "e2e".into(),
        category,
        name: name.into(),
        version: None,
        description: None,
        catalog_id: None,
        origin: Origin::Command("CI restore fixture".into()),
        portability,
        details,
        findings: vec![],
    };
    let items = vec![
        item(
            package,
            Category::PackageManager,
            ItemDetails::Package { manager, kind },
            Portability::SameOs,
        ),
        item(
            MISSING,
            Category::PackageManager,
            ItemDetails::Package { manager, kind },
            Portability::SameOs,
        ),
        item(
            "is-odd",
            Category::GlobalPackage,
            ItemDetails::GlobalPackage {
                manager: GlobalManager::Npm,
            },
            Portability::Any,
        ),
        item(
            ".repot-e2e",
            Category::Shell,
            ItemDetails::ConfigFile {
                source,
                target: TargetPath::Home(".repot-e2e".into()),
            },
            Portability::Any,
        ),
    ];
    let selection = Selection {
        items: items.iter().map(|i| i.id.clone()).collect(),
        reveal_secret_lines: vec![],
    };
    build_export(
        &env,
        &ExportRequest {
            items: &items,
            selection: &selection,
            source: os,
            target: repot_core::Target::default_for(os),
            format: Format::Script,
            meta: ExportMeta::now(),
        },
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    let os = match args.get(1).map(String::as_str) {
        Some("macos") => Os::MacOs,
        Some("ubuntu") => Os::Linux(LinuxFamily::Debian),
        Some("windows") => Os::Windows,
        _ => return Err("usage: e2e_export <macos|ubuntu|windows> <out>".into()),
    };
    let parent = PathBuf::from(args.get(2).ok_or("missing output directory")?);
    std::fs::create_dir_all(&parent)?;
    let bundle = e2e_bundle(os);
    if !bundle.warnings.is_empty() {
        return Err(format!("unexpected export warnings: {:?}", bundle.warnings).into());
    }
    println!(
        "{}",
        write_export(&bundle, &parent, &ExportMeta::now().date)?.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_targets_export_real_packages_one_deliberate_failure_and_config() {
        for os in [Os::MacOs, Os::Windows, Os::Linux(LinuxFamily::Debian)] {
            let bundle = e2e_bundle(os);
            assert!(bundle.warnings.is_empty());
            assert_eq!(bundle.files.len(), 3); // script, repot.json, payload
            let payload = bundle
                .files
                .iter()
                .find(|f| f.path == "files/home/.repot-e2e")
                .unwrap();
            assert_eq!(payload.contents, CONTENT);
            let script = &bundle
                .files
                .iter()
                .find(|f| f.path.starts_with("restore."))
                .unwrap()
                .contents;
            assert!(script.contains(MISSING));
            assert!(script.contains("is-odd"));
            assert!(script.contains(if os == Os::Windows {
                "dos2unix"
            } else {
                "'sl'"
            }));
            assert_eq!(script.matches("FAILED: ").count(), 1);
        }
    }
}
