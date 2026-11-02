use repot_core::{
    env::FakeEnv,
    os::{LinuxFamily, Os},
    providers::pacman::Pacman,
    scan::Provider,
};
#[test]
fn explicit_packages_with_local_db_descriptions() {
    let env = FakeEnv::new(Os::Linux(LinuxFamily::Arch), "/home/t")
        .with_command("pacman", &["-Qe"], "git 2.47.0-1\nripgrep 14.1.1-1\n")
        .with_file("/var/lib/pacman/local/git-2.47.0-1/desc", "%NAME%\ngit\n\n%DESC%\nthe fast distributed version control system\n\n%URL%\nhttps://git-scm.com/\n");
    let items = Pacman.scan(&env).unwrap();
    assert_eq!(
        items
            .iter()
            .map(|i| (i.id.as_str(), i.version.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            ("pacman:git", Some("2.47.0-1")),
            ("pacman:ripgrep", Some("14.1.1-1"))
        ]
    );
    assert_eq!(
        items[0].description.as_deref(),
        Some("the fast distributed version control system")
    );
    assert_eq!(items[1].description, None);
}

#[test]
fn malformed_line_fails() {
    let env = FakeEnv::new(Os::Linux(LinuxFamily::Arch), "/home/t").with_command(
        "pacman",
        &["-Qe"],
        "git\n",
    );
    assert!(matches!(
        Pacman.scan(&env),
        Err(repot_core::scan::ProviderError::Parse { .. })
    ));
}
