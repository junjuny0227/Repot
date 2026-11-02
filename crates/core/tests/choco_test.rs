use repot_core::{env::FakeEnv, os::Os, providers::choco::Choco, scan::Provider};
fn nuspec(summary: &str, deps: &[&str]) -> String {
    let d: String = deps
        .iter()
        .map(|id| format!(r#"<dependency id="{id}" version="1.0" />"#))
        .collect();
    format!(
        r#"<?xml version="1.0"?><package><metadata><summary>{summary}</summary><dependencies>{d}</dependencies></metadata></package>"#
    )
}

#[test]
fn fallback_description_case_insensitive_dependencies_and_missing_nuspec() {
    let env = FakeEnv::new(Os::Windows, "C:/Users/t")
        .with_var("ChocolateyInstall", "D:/Choco")
        .with_command(
            "choco",
            &["list", "--limit-output"],
            "git|1
git.install|1
unknown|2
",
        )
        .with_file(
            "D:/Choco/lib/git/git.nuspec",
            "<description>Git &lt;tools&gt; &quot;ok&quot; &apos;x&apos; &amp; more
second</description><dependency id='GIT.INSTALL'/>",
        );
    let items = Choco.scan(&env).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(
        items[0].description.as_deref(),
        Some("Git <tools> \"ok\" 'x' & more")
    );
    assert_eq!(items[1].description, None);
}

#[test]
fn malformed_line_fails() {
    let env = FakeEnv::new(Os::Windows, "C:/Users/t").with_command(
        "choco",
        &["list", "--limit-output"],
        "git|1\nbad\n",
    );
    assert!(matches!(
        Choco.scan(&env),
        Err(repot_core::scan::ProviderError::Parse { .. })
    ));
}

#[test]
fn drops_dependency_packages() {
    // Review Focus 2
    let lib = "C:/ProgramData/chocolatey/lib";
    let env = FakeEnv::new(Os::Windows, "C:/Users/t")
        .with_command(
            "choco",
            &["list", "--limit-output"],
            "chocolatey|2.3.0\ngit|2.47.0\ngit.install|2.47.0\n7zip|24.8.0\n7zip.install|24.8.0\n",
        )
        .with_file(
            format!("{lib}/git/git.nuspec"),
            &nuspec("Git (Install)", &["git.install"]),
        )
        .with_file(
            format!("{lib}/git.install/git.install.nuspec"),
            &nuspec("Git (Install)", &[]),
        )
        .with_file(
            format!("{lib}/7zip/7zip.nuspec"),
            &nuspec("7-Zip &amp; tools", &["7zip.install"]),
        )
        .with_file(
            format!("{lib}/7zip.install/7zip.install.nuspec"),
            &nuspec("7-Zip", &[]),
        );
    let items = Choco.scan(&env).unwrap();
    assert_eq!(
        items.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        vec!["choco:git", "choco:7zip"]
    );
    assert_eq!(items[1].description.as_deref(), Some("7-Zip & tools"));
    assert_eq!(items[0].version.as_deref(), Some("2.47.0"));
}
