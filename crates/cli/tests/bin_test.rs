//! The built `repot` binary: argument handling only. Commands are tested in run_test.rs
//! with a fake machine; nothing here scans the real one.
use std::process::Command;

fn repot(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_repot"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn help_lists_the_commands() {
    let o = repot(&["--help"]);
    assert!(o.status.success());
    let text = String::from_utf8_lossy(&o.stdout);
    for command in ["scan", "compare", "export"] {
        assert!(text.contains(command), "{text}");
    }
    let export = String::from_utf8_lossy(&repot(&["export", "--help"]).stdout).into_owned();
    assert!(export.contains("--only-missing"), "{export}");
}

#[test]
fn version_matches_the_crate() {
    let o = repot(&["--version"]);
    assert!(o.status.success());
    assert_eq!(
        String::from_utf8_lossy(&o.stdout),
        format!("repot {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn bad_arguments_exit_with_2() {
    for args in [&["export", "--to", "plan9"][..], &["unknown"], &[]] {
        let o = repot(args);
        assert_eq!(o.status.code(), Some(2), "{args:?}");
    }
}
