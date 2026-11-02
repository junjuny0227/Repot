use repot_core::export::{
    ExportBundle, OutputFile,
    write::{ExportError, write_export},
};
fn bundle(paths: &[&str]) -> ExportBundle {
    ExportBundle {
        warnings: vec![],
        files: paths
            .iter()
            .map(|p| OutputFile {
                path: p.to_string(),
                contents: format!("content of {p}"),
                executable: p.ends_with(".sh"),
            })
            .collect(),
    }
}
#[test]
fn writes_and_numbers_folders() {
    let dir = tempfile::tempdir().unwrap();
    let b = bundle(&["restore.sh", "files/home/.zshrc"]);
    let first = write_export(&b, dir.path(), "2026-11-02").unwrap();
    assert_eq!(first, dir.path().join("repot-export-2026-11-02"));
    assert_eq!(
        std::fs::read_to_string(first.join("files/home/.zshrc")).unwrap(),
        "content of files/home/.zshrc"
    );
    let second = write_export(&b, dir.path(), "2026-11-02").unwrap();
    assert_eq!(second, dir.path().join("repot-export-2026-11-02-2"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(first.join("restore.sh"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o755
        );
    }
}
#[test]
fn failed_write_leaves_nothing() {
    let dir = tempfile::tempdir().unwrap();
    for bad in [
        "../escape.txt",
        "/abs.txt",
        "",
        r"..\escape.txt",
        r"C:\escape.txt",
        r"\\server\share\file",
        "a/../b",
    ] {
        assert!(
            matches!(
                write_export(&bundle(&["README.md", bad]), dir.path(), "2026-11-02"),
                Err(ExportError::InvalidPath(_))
            ),
            "{bad}"
        );
    }
    assert!(matches!(
        write_export(&bundle(&["a", "a/b"]), dir.path(), "2026-11-02"),
        Err(ExportError::Io(_))
    ));
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}
#[test]
fn invalid_date_and_paths_do_not_touch_disk() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing");
    assert!(matches!(
        write_export(&bundle(&["../bad"]), &missing, "2026-11-02"),
        Err(ExportError::InvalidPath(_))
    ));
    assert!(!missing.exists());
    for date in ["", "../../escape", "2026/11/02"] {
        assert!(matches!(
            write_export(&bundle(&["README.md"]), dir.path(), date),
            Err(ExportError::InvalidPath(_))
        ));
    }
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}
#[test]
fn existing_exports_and_files_are_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("repot-export-2026-11-02");
    std::fs::write(&first, "keep").unwrap();
    std::fs::create_dir(dir.path().join("repot-export-2026-11-02-2")).unwrap();
    let third = write_export(&bundle(&["README.md"]), dir.path(), "2026-11-02").unwrap();
    assert_eq!(third.file_name().unwrap(), "repot-export-2026-11-02-3");
    assert_eq!(std::fs::read_to_string(first).unwrap(), "keep");
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 3);
}
