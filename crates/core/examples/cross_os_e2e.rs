//! A macOS scan exported for an actual Windows winget restore in CI.
use repot_core::{
    FakeEnv, Target,
    export::{ExportBundle, ExportMeta, ExportRequest, Format, build_export, write_export},
    model::*,
    os::Os,
};

fn bundle() -> ExportBundle {
    let items: Vec<_> = [
        ("ripgrep", PackageKind::Formula),
        ("rectangle", PackageKind::Cask),
        ("repot-unknown-fixture", PackageKind::Formula),
    ]
    .into_iter()
    .map(|(name, kind)| Item {
        id: format!("brew:{kind:?}:{name}"),
        provider: "brew".into(),
        category: Category::PackageManager,
        name: name.into(),
        version: None,
        description: None,
        catalog_id: None,
        origin: Origin::Command("macOS cross-OS CI fixture".into()),
        portability: Portability::Package,
        details: ItemDetails::Package {
            manager: PackageManager::Brew,
            kind,
        },
        findings: vec![],
    })
    .collect();
    build_export(
        &FakeEnv::new(Os::MacOs, "/repot-fixture"),
        &ExportRequest {
            selection: &Selection {
                items: items.iter().map(|i| i.id.clone()).collect(),
                reveal_secret_lines: vec![],
            },
            items: &items,
            source: Os::MacOs,
            target: Target::default_for(Os::Windows),
            format: Format::Script,
            meta: ExportMeta::now(),
        },
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parent = std::env::args().nth(1).ok_or("usage: cross_os_e2e <out>")?;
    let bundle = bundle();
    if bundle.warnings.len() != 2 {
        return Err(format!("unexpected warnings: {:?}", bundle.warnings).into());
    }
    println!(
        "{}",
        write_export(
            &bundle,
            std::path::Path::new(&parent),
            &ExportMeta::now().date
        )?
        .display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_package_is_mapped_and_unknown_or_absent_packages_are_excluded() {
        let bundle = bundle();
        assert_eq!(bundle.warnings.len(), 2);
        assert!(matches!(
            &bundle.warnings[0],
            repot_core::export::ExportWarning::Unavailable {
                reason: repot_core::availability::UnavailableReason::NoPackageInManager { .. },
                ..
            }
        ));
        assert!(matches!(
            &bundle.warnings[1],
            repot_core::export::ExportWarning::Unavailable {
                reason: repot_core::availability::UnavailableReason::NotInPackageMap { .. },
                ..
            }
        ));
        let script = &bundle
            .files
            .iter()
            .find(|f| f.path == "restore.ps1")
            .unwrap()
            .contents;
        assert!(script.contains("# from brew: ripgrep"));
        assert!(script.contains("'BurntSushi.ripgrep.MSVC'"));
        assert!(!script.contains("rectangle"));
        assert!(!script.contains("repot-unknown-fixture"));
        let readme = &bundle
            .files
            .iter()
            .find(|f| f.path == "README.md")
            .unwrap()
            .contents;
        assert!(readme.contains("ripgrep"));
    }
}
