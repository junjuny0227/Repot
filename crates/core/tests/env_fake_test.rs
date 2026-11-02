use repot_core::env::{CommandOutput, Env, EnvError, FakeEnv};
use repot_core::os::Os;
use std::path::{Path, PathBuf};

#[test]
fn fake_env_basics() {
    let env = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_command("brew", &["tap"], "user/tools\n")
        .with_file("/Users/t/.zshrc", "x")
        .with_file("/Users/t/.config/fish/config.fish", "y")
        .with_unreadable_file("/Users/t/.bashrc");
    assert!(env.which("brew"));
    assert!(!env.which("npm"));
    assert_eq!(env.run("brew", &["tap"]).unwrap().stdout, "user/tools\n");
    assert_eq!(env.run("brew", &["list"]), Err(EnvError::NotFound));
    assert_eq!(
        env.read_file(Path::new("/Users/t/.bashrc")),
        Err(EnvError::PermissionDenied)
    );
    assert_eq!(
        env.read_file(Path::new("/Users/t/nope")),
        Err(EnvError::NotFound)
    );
    assert!(env.exists(Path::new("/Users/t/.config")));
    assert_eq!(
        env.list_dir(Path::new("/Users/t")).unwrap(),
        vec![
            PathBuf::from("/Users/t/.bashrc"),
            PathBuf::from("/Users/t/.config"),
            PathBuf::from("/Users/t/.zshrc")
        ]
    );
}

#[test]
fn fake_environment_values_and_outputs() {
    let env = FakeEnv::new(Os::Windows, "C:/Users/홍 길동")
        .with_var("APPDATA", "custom")
        .with_program("git")
        .with_command_output(
            "npm",
            &["ls"],
            Ok(CommandOutput {
                code: Some(1),
                stdout: "{}".into(),
                stderr: "warning".into(),
            }),
        )
        .with_command_output("slow", &[], Err(EnvError::Timeout));
    assert_eq!(env.os(), Os::Windows);
    assert_eq!(env.home_dir(), PathBuf::from("C:/Users/홍 길동"));
    assert_eq!(env.var("APPDATA").as_deref(), Some("custom"));
    assert_eq!(env.var("MISSING"), None);
    assert!(env.which("git") && env.which("npm") && env.which("slow"));
    assert!(!env.run("npm", &["ls"]).unwrap().success());
    assert_eq!(env.run("slow", &[]), Err(EnvError::Timeout));
    assert_eq!(env.list_dir(Path::new("missing")), Err(EnvError::NotFound));
    assert!(
        !CommandOutput {
            code: None,
            stdout: String::new(),
            stderr: String::new()
        }
        .success()
    );
}
