use repot_core::os::{LinuxFamily, Os, Shell, current_os, parse_os_release};
#[test]
fn os_release_families() {
    for (text, family) in [
        ("ID=ubuntu\nID_LIKE=debian\n", LinuxFamily::Debian),
        ("ID=\"fedora\"\n", LinuxFamily::Fedora),
        (
            "ID=rocky\nID_LIKE=\"rhel centos fedora\"\n",
            LinuxFamily::Fedora,
        ),
        ("ID=endeavouros\nID_LIKE=arch\n", LinuxFamily::Arch),
        ("ID=nixos\n", LinuxFamily::Other),
        ("", LinuxFamily::Other),
        ("ID=arch\nID_LIKE=debian\n", LinuxFamily::Arch),
        (
            "ID=unknown\nID_LIKE='unknown ubuntu'\n",
            LinuxFamily::Debian,
        ),
    ] {
        assert_eq!(parse_os_release(text), family);
    }
}
#[test]
fn shell_support() {
    for os in [Os::MacOs, Os::Windows, Os::Linux(LinuxFamily::Arch)] {
        for shell in [Shell::Zsh, Shell::Bash, Shell::Fish] {
            assert_eq!(os.supports_shell(shell), os != Os::Windows);
        }
        assert!(os.supports_shell(Shell::PowerShell));
    }
}
#[test]
fn os_serializes_camel_case() {
    assert_eq!(serde_json::to_string(&Os::MacOs).unwrap(), "\"macOs\"");
    assert_eq!(
        serde_json::to_string(&Os::Linux(LinuxFamily::Debian)).unwrap(),
        "{\"linux\":\"debian\"}"
    );
}
#[test]
fn current_os_matches_target() {
    let os = current_os();
    if cfg!(target_os = "macos") {
        assert_eq!(os, Os::MacOs);
    } else if cfg!(target_os = "windows") {
        assert_eq!(os, Os::Windows);
    } else {
        assert!(matches!(os, Os::Linux(_)));
    }
}
