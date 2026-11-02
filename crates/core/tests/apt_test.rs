use repot_core::{
    env::FakeEnv,
    os::{LinuxFamily, Os},
    providers::apt::Apt,
    scan::Provider,
};
const DPKG_ARGS: &[&str] = &["-W", "-f=${Package}\\t${Version}\\t${binary:Summary}\\n"];

fn env(with_initial: bool) -> FakeEnv {
    let mut e = FakeEnv::new(Os::Linux(LinuxFamily::Debian), "/home/t")
        .with_command("apt-mark", &["showmanual"], "curl\nripgrep\nbase-files\n")
        .with_command("dpkg-query", DPKG_ARGS,
            "curl\t8.5.0\tcommand line tool for transferring data with URL syntax\nripgrep\t14.1.0\tRecursively searches directories for a regex pattern\nbase-files\t13\tDebian base system miscellaneous files\n");
    if with_initial {
        e = e
            .with_file("/var/log/installer/initial-status.gz", "<gz>")
            .with_command(
                "gzip",
                &["-dc", "/var/log/installer/initial-status.gz"],
                "Package: base-files\nStatus: install ok installed\n\nPackage: curl\n",
            );
    }
    e
}

#[test]
fn filters_initial_packages() {
    let ids: Vec<_> = Apt
        .scan(&env(true))
        .unwrap()
        .into_iter()
        .map(|i| i.id)
        .collect();
    assert_eq!(ids, vec!["apt:ripgrep"]);
}

#[test]
fn no_initial_status_is_not_an_error() {
    // Review Focus 3
    let items = Apt.scan(&env(false)).unwrap();
    assert_eq!(
        items.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        vec!["apt:base-files", "apt:curl", "apt:ripgrep"]
    );
    assert_eq!(
        items[1].description.as_deref(),
        Some("command line tool for transferring data with URL syntax")
    );
}

#[test]
fn arch_names_and_failed_gzip() {
    let env = FakeEnv::new(Os::Linux(LinuxFamily::Debian), "/home/t")
        .with_command("apt-mark", &["showmanual"], "curl:amd64\n")
        .with_command("dpkg-query", DPKG_ARGS, "curl\t8.5\tSummary\n")
        .with_file("/var/log/installer/initial-status.gz", "gzip");
    let items = Apt.scan(&env).unwrap();
    assert_eq!(items[0].id, "apt:curl:amd64");
    assert_eq!(items[0].version.as_deref(), Some("8.5"));
}
#[test]
fn malformed_metadata_fails() {
    let env = FakeEnv::new(Os::Linux(LinuxFamily::Debian), "/home/t")
        .with_command("apt-mark", &["showmanual"], "curl\n")
        .with_command("dpkg-query", DPKG_ARGS, "malformed\n");
    assert!(matches!(
        Apt.scan(&env),
        Err(repot_core::scan::ProviderError::Parse { .. })
    ));
}
