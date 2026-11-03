use repot_core::{
    env::FakeEnv,
    model::*,
    os::{LinuxFamily, Os},
    providers::nvm::Nvm,
    scan::Provider,
};
fn default_of(item: &Item) -> bool {
    matches!(
        item.details,
        ItemDetails::Runtime {
            is_default: true,
            ..
        }
    )
}
fn env_with(default: &str) -> FakeEnv {
    FakeEnv::new(Os::MacOs, "/Users/t")
        .with_file("/Users/t/.nvm/versions/node/v18.20.4/bin/node", "")
        .with_file("/Users/t/.nvm/versions/node/v20.9.0/bin/node", "")
        .with_file("/Users/t/.nvm/versions/node/v20.11.0/bin/node", "")
        .with_file("/Users/t/.nvm/alias/default", default)
}

#[test]
fn lists_versions_and_resolves_major_default() {
    let items = Nvm.scan(&env_with("20\n")).unwrap();
    let v: Vec<_> = items
        .iter()
        .map(|i| (i.version.clone().unwrap(), default_of(i)))
        .collect();
    assert_eq!(
        v,
        vec![
            ("18.20.4".into(), false),
            ("20.9.0".into(), false),
            ("20.11.0".into(), true)
        ]
    );
    assert_eq!(items[0].portability, Portability::Mapped(RuntimeTool::Nvm));
    assert_eq!(items[0].catalog_id.as_deref(), Some("runtime:node"));
}

#[test]
fn exact_and_alias_defaults() {
    assert!(default_of(&Nvm.scan(&env_with("v18.20.4")).unwrap()[0]));
    assert!(
        Nvm.scan(&env_with("lts/*"))
            .unwrap()
            .iter()
            .all(|i| !default_of(i))
    );
}

#[test]
fn respects_nvm_dir() {
    let env = FakeEnv::new(Os::Linux(LinuxFamily::Debian), "/home/t")
        .with_var("NVM_DIR", "/opt/nvm")
        .with_file("/opt/nvm/versions/node/v22.0.0/bin/node", "");
    assert!(Nvm.detect(&env));
    assert_eq!(Nvm.scan(&env).unwrap()[0].id, "nvm:node:22.0.0");
}
