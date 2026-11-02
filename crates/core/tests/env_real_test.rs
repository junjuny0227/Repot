use repot_core::env::real::{RealEnv, candidate_names};
use repot_core::env::{Env, EnvError};
use std::time::{Duration, Instant};

#[test]
fn candidate_names_uses_pathext() {
    assert_eq!(
        candidate_names("npm", Some(".COM;.EXE;.BAT;.CMD")),
        vec!["npm.com", "npm.exe", "npm.bat", "npm.cmd"]
    );
    assert_eq!(candidate_names("npm", None), vec!["npm"]);
    assert_eq!(
        candidate_names("npm.cmd", Some(".EXE;.CMD")),
        vec!["npm.cmd"]
    );
}

#[test]
fn runs_real_command() {
    let env = RealEnv::new();
    assert!(env.which("cargo"));
    let out = env.run("cargo", &["--version"]).unwrap();
    assert!(out.success());
    assert!(out.stdout.starts_with("cargo "));
    assert_eq!(
        env.run("definitely-not-a-program-repot", &[]),
        Err(EnvError::NotFound)
    );
}

#[cfg(unix)]
#[test]
fn command_timeout() {
    let env = RealEnv::new().with_timeout(Duration::from_millis(200));
    let start = Instant::now();
    assert_eq!(env.run("sleep", &["5"]), Err(EnvError::Timeout));
    assert!(start.elapsed() < Duration::from_secs(2));
}

#[cfg(unix)]
#[test]
fn drains_both_pipes_and_preserves_failure_output() {
    let env = RealEnv::new();
    let out = env.run("sh", &["-c", "i=0; while [ $i -lt 20000 ]; do printf 1234567890; printf abcdefghij >&2; i=$((i+1)); done; exit 7"]).unwrap();
    assert_eq!(out.code, Some(7));
    assert_eq!(out.stdout.len(), 200000);
    assert_eq!(out.stderr.len(), 200000);
}

#[cfg(unix)]
#[test]
fn inherited_pipes_do_not_extend_timeout() {
    let env = RealEnv::new().with_timeout(Duration::from_millis(200));
    let start = Instant::now();
    assert_eq!(
        env.run("sh", &["-c", "sleep 2 & exit 0"]),
        Err(EnvError::Timeout)
    );
    assert!(start.elapsed() < Duration::from_secs(1));
}

#[cfg(unix)]
#[test]
fn resolves_absolute_paths_with_spaces_and_rejects_non_executables() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("홍 길동 tool");
    std::fs::write(&path, "#!/bin/sh\nprintf '%s' \"$PATH\"\n").unwrap();
    let env = RealEnv::new();
    assert!(!env.which(path.to_str().unwrap()));
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(env.which(path.to_str().unwrap()));
    assert_eq!(
        env.run(path.to_str().unwrap(), &[]).unwrap().stdout,
        env.var("PATH").unwrap()
    );
}

#[test]
fn reads_files_and_dirs() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("b.txt"), [b'h', 0xff]).unwrap();
    std::fs::create_dir(dir.path().join("a")).unwrap();
    let env = RealEnv::new();
    assert_eq!(env.read_file(&dir.path().join("b.txt")).unwrap(), "h�");
    assert_eq!(
        env.list_dir(dir.path()).unwrap(),
        vec![dir.path().join("a"), dir.path().join("b.txt")]
    );
    assert_eq!(
        env.read_file(&dir.path().join("none")),
        Err(EnvError::NotFound)
    );
    assert!(env.exists(dir.path()));
    assert!(!env.exists(&dir.path().join("none")));
    assert!(!env.home_dir().as_os_str().is_empty());
    assert!(env.var("PATH").is_some());
}
