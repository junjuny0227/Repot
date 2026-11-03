use repot_core::{
    model::*,
    package_map::{Lookup, PackageMap},
};

#[test]
fn embedded_data_and_lookup_distinguish_missing_and_null() {
    let map = PackageMap::parse(include_str!("../data/package-map.json")).unwrap();
    assert_eq!(
        map.lookup(
            PackageManager::Brew,
            PackageKind::Formula,
            "RIPGREP",
            PackageManager::Winget
        ),
        Lookup::Found {
            kind: PackageKind::Package,
            name: "BurntSushi.ripgrep.MSVC"
        }
    );
    assert_eq!(
        map.lookup(
            PackageManager::Brew,
            PackageKind::Cask,
            "rectangle",
            PackageManager::Winget
        ),
        Lookup::Absent
    );
    assert_eq!(
        map.lookup(
            PackageManager::Brew,
            PackageKind::Formula,
            "unknown",
            PackageManager::Winget
        ),
        Lookup::Missing
    );
    assert_eq!(
        map.lookup(
            PackageManager::Brew,
            PackageKind::Formula,
            "owner/tap/ripgrep",
            PackageManager::Winget
        ),
        Lookup::Missing
    );
    assert_eq!(
        map.lookup(
            PackageManager::Brew,
            PackageKind::Formula,
            "firefox",
            PackageManager::Winget
        ),
        Lookup::Missing
    );
    let small = PackageMap::parse(
        r#"{"version":1,"packages":{"example":{"brew":"example","scoop":"extras/example"}}}"#,
    )
    .unwrap();
    assert_eq!(
        small.lookup(
            PackageManager::Brew,
            PackageKind::Formula,
            "example",
            PackageManager::Apt
        ),
        Lookup::Missing
    );
    assert_eq!(
        small.lookup(
            PackageManager::Scoop,
            PackageKind::Package,
            "extras/example",
            PackageManager::Brew
        ),
        Lookup::Found {
            kind: PackageKind::Formula,
            name: "example"
        }
    );
}

#[test]
fn invalid_tables_are_rejected() {
    for data in [
        r#"{"version":2,"packages":{}}"#,
        r#"{"version":1,"packages":{"x":{"unknown":"x"}}}"#,
        r#"{"version":1,"packages":{"z":{"brew":"z"},"a":{"brew":"a"}}}"#,
        r#"{"version":1,"packages":{"Bad":{"brew":"a"}}}"#,
        r#"{"version":1,"packages":{"x":{"brew":""}}}"#,
        r#"{"version":1,"packages":{"x":{"brew":"a b"}}}"#,
        r#"{"version":1,"packages":{"a":{"winget":"Foo.Bar"},"b":{"winget":"foo.bar"}}}"#,
        r#"{"version":1,"packages":{"a":{"scoop":"foo"},"b":{"scoop":"main/foo"}}}"#,
        r#"{"version":1,"packages":{"x":{"scoop":"community/foo"}}}"#,
        r#"{"version":1,"packages":{"x":{"scoop":"extras/foo/bar"}}}"#,
    ] {
        assert!(PackageMap::parse(data).is_err(), "accepted {data}");
    }
}
