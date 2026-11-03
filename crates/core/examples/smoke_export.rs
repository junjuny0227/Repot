use repot_core::{
    FakeEnv,
    export::{ExportBundle, ExportMeta, ExportRequest, Format, build_export, write_export},
    model::*,
    os::{Os, current_os},
};
use std::{error::Error, path::PathBuf};

fn smoke_bundle(os: Os) -> ExportBundle {
    let source = PathBuf::from("/repot-fixture/.repot-smoke");
    let env = FakeEnv::new(os, "/repot-fixture").with_file(&source, "smoke\n");
    let items = vec![
        Item {
            id: "smoke:file".into(),
            provider: "smoke".into(),
            category: Category::Shell,
            name: ".repot-smoke".into(),
            version: None,
            description: None,
            catalog_id: None,
            origin: Origin::Path(source.clone()),
            portability: Portability::Any,
            details: ItemDetails::ConfigFile {
                source,
                target: TargetPath::Home(".repot-smoke".into()),
            },
            findings: vec![],
        },
        Item {
            id: "smoke:npm".into(),
            provider: "npm".into(),
            category: Category::GlobalPackage,
            name: "cowsay".into(),
            version: None,
            description: None,
            catalog_id: None,
            origin: Origin::Command("npm ls -g".into()),
            portability: Portability::Any,
            details: ItemDetails::GlobalPackage {
                manager: GlobalManager::Npm,
            },
            findings: vec![],
        },
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
    let parent = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("usage: smoke_export <out>")?,
    );
    std::fs::create_dir_all(&parent)?;
    let bundle = smoke_bundle(current_os());
    if !bundle.warnings.is_empty() {
        return Err(format!("unexpected export warnings: {:?}", bundle.warnings).into());
    }
    let path = write_export(&bundle, &parent, &ExportMeta::now().date)?;
    println!("{}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use repot_core::os::LinuxFamily;

    #[test]
    fn smoke_bundle_has_one_copy_and_npm_without_warnings() {
        for os in [Os::MacOs, Os::Windows, Os::Linux(LinuxFamily::Debian)] {
            let bundle = smoke_bundle(os);
            assert!(bundle.warnings.is_empty());
            assert_eq!(bundle.files.len(), 2);
            let payload = bundle
                .files
                .iter()
                .find(|f| f.path == "files/home/.repot-smoke")
                .unwrap();
            assert_eq!(payload.contents, "smoke\n");
            let script = bundle
                .files
                .iter()
                .find(|f| f.path.starts_with("restore."))
                .unwrap();
            assert!(script.contents.contains("cowsay"));
            assert!(
                script.contents.contains("copy ~/.repot-smoke")
                    || script.contents.contains("copy %USERPROFILE%\\.repot-smoke")
            );
        }
    }
}
