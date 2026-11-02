use repot_core::{
    env::FakeEnv,
    model::*,
    os::{LinuxFamily, Os},
    providers::sdkman::Sdkman,
    scan::Provider,
};

fn is_default(item: &Item) -> bool {
    matches!(
        item.details,
        ItemDetails::Runtime {
            is_default: true,
            ..
        }
    )
}

#[test]
fn candidates_and_current_link() {
    let c = "/Users/t/.sdkman/candidates";
    let env = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_file(format!("{c}/java/17.0.13-tem/bin/java"), "")
        .with_file(format!("{c}/java/21.0.5-tem/bin/java"), "")
        .with_link(format!("{c}/java/current"), format!("{c}/java/21.0.5-tem"))
        .with_file(format!("{c}/gradle/8.11/bin/gradle"), "");
    assert!(Sdkman.detect(&env));
    let items = Sdkman.scan(&env).unwrap();
    assert_eq!(
        items
            .iter()
            .map(|i| (i.id.as_str(), is_default(i)))
            .collect::<Vec<_>>(),
        vec![
            ("sdkman:gradle:8.11", false),
            ("sdkman:java:17.0.13-tem", false),
            ("sdkman:java:21.0.5-tem", true)
        ]
    );
    assert_eq!(items[0].name, "gradle");
    assert_eq!(
        items[0].portability,
        Portability::Mapped(RuntimeTool::Sdkman)
    );
}

#[test]
fn respects_sdkman_dir_and_relative_link() {
    let c = "/opt/sdkman/candidates";
    let env = FakeEnv::new(Os::Linux(LinuxFamily::Debian), "/home/t")
        .with_var("SDKMAN_DIR", "/opt/sdkman")
        .with_file(format!("{c}/maven/3.9.9/bin/mvn"), "")
        .with_link(format!("{c}/maven/current"), "3.9.9");
    assert!(Sdkman.detect(&env));
    let items = Sdkman.scan(&env).unwrap();
    assert_eq!(items[0].id, "sdkman:maven:3.9.9");
    assert!(is_default(&items[0]));
    assert!(!Sdkman.detect(&FakeEnv::new(Os::MacOs, "/Users/t")));
}

#[test]
fn skips_stray_files() {
    let c = "/Users/t/.sdkman/candidates";
    let env = FakeEnv::new(Os::MacOs, "/Users/t")
        .with_file(format!("{c}/.DS_Store"), "")
        .with_file(format!("{c}/java/.DS_Store"), "")
        .with_file(format!("{c}/java/21.0.5-tem/bin/java"), "");
    let ids: Vec<_> = Sdkman
        .scan(&env)
        .unwrap()
        .into_iter()
        .map(|i| i.id)
        .collect();
    assert_eq!(ids, vec!["sdkman:java:21.0.5-tem"]);
}
