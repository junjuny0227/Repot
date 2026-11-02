use repot_core::{
    env::FakeEnv,
    os::{LinuxFamily, Os},
    providers::dnf::Dnf,
    scan::{Provider, ProviderError},
};
const ARGS: &[&str] = &[
    "repoquery",
    "--userinstalled",
    "--qf",
    "%{name}|%{version}|%{summary}\\n",
];

#[test]
fn parses_and_dedupes() {
    let env = FakeEnv::new(Os::Linux(LinuxFamily::Fedora), "/home/t").with_command("dnf", ARGS,
        "ripgrep|14.1.1|Line-oriented search tool\nglibc|2.40|The GNU libc libraries\nglibc|2.40|The GNU libc libraries\ngh|2.62.0|GitHub's official command line tool | CLI\n");
    let items = Dnf.scan(&env).unwrap();
    assert_eq!(
        items.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        vec!["dnf:gh", "dnf:glibc", "dnf:ripgrep"]
    );
    assert_eq!(
        items[0].description.as_deref(),
        Some("GitHub's official command line tool | CLI")
    );
}

#[test]
fn malformed_line_fails() {
    let env = FakeEnv::new(Os::Linux(LinuxFamily::Fedora), "/home/t").with_command(
        "dnf",
        ARGS,
        "ripgrep-14.1.1.x86_64\n",
    );
    assert!(matches!(Dnf.scan(&env), Err(ProviderError::Parse { .. })));
}
