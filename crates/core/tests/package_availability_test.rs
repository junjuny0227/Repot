use repot_core::{
    availability::{Availability, UnavailableReason, availability},
    os::{LinuxFamily, Os},
    *,
};
fn package(manager: PackageManager, kind: PackageKind, name: &str) -> Item {
    Item {
        id: name.into(),
        provider: "fixture".into(),
        category: Category::PackageManager,
        name: name.into(),
        version: None,
        description: None,
        catalog_id: None,
        origin: Origin::Command("fixture".into()),
        portability: Portability::Package,
        details: ItemDetails::Package { manager, kind },
        findings: vec![],
    }
}
#[test]
fn packages_map_only_explicit_names_across_os_and_linux_families() {
    use PackageManager::*;
    let item = package(Brew, PackageKind::Formula, "ripgrep");
    for (os, manager, name) in [
        (Os::Windows, Winget, "BurntSushi.ripgrep.MSVC"),
        (Os::Windows, Scoop, "ripgrep"),
        (Os::Windows, Choco, "ripgrep"),
        (Os::Linux(LinuxFamily::Debian), Apt, "ripgrep"),
        (Os::Linux(LinuxFamily::Fedora), Dnf, "ripgrep"),
        (Os::Linux(LinuxFamily::Arch), Pacman, "ripgrep"),
    ] {
        assert_eq!(
            availability(
                &item,
                Os::MacOs,
                Target {
                    os,
                    packages: Some(manager)
                }
            ),
            Availability::Mapped {
                manager,
                name: name.into()
            }
        );
    }
    let debian = package(Apt, PackageKind::Package, "ripgrep");
    assert_eq!(
        availability(
            &debian,
            Os::Linux(LinuxFamily::Debian),
            Target::default_for(Os::Linux(LinuxFamily::Fedora))
        ),
        Availability::Mapped {
            manager: Dnf,
            name: "ripgrep".into()
        }
    );
    let firefox = package(Brew, PackageKind::Cask, "firefox");
    assert_eq!(
        availability(
            &firefox,
            Os::MacOs,
            Target::default_for(Os::Linux(LinuxFamily::Debian))
        ),
        Availability::Unavailable(UnavailableReason::NoPackageInManager { manager: Apt })
    );
    for name in ["missing", "owner/tap/ripgrep"] {
        assert_eq!(
            availability(
                &package(Brew, PackageKind::Formula, name),
                Os::MacOs,
                Target::default_for(Os::Windows)
            ),
            Availability::Unavailable(UnavailableReason::NotInPackageMap { manager: Winget })
        );
    }
    assert_eq!(
        availability(
            &package(Brew, PackageKind::Cask, "rectangle"),
            Os::MacOs,
            Target::default_for(Os::Windows)
        ),
        Availability::Unavailable(UnavailableReason::NoPackageInManager { manager: Winget })
    );
}
#[test]
fn same_os_preserves_original_manager_and_unknown_packages() {
    let item = package(PackageManager::Scoop, PackageKind::Package, "unknown");
    assert_eq!(
        availability(
            &item,
            Os::Windows,
            Target {
                os: Os::Windows,
                packages: Some(PackageManager::Choco)
            }
        ),
        Availability::Available
    );
    let other = Os::Linux(LinuxFamily::Other);
    assert_eq!(
        availability(&item, other, Target::default_for(other)),
        Availability::Available
    );
}
#[test]
fn taps_other_linux_and_unreadable_are_unavailable() {
    let mut item = package(PackageManager::Brew, PackageKind::Tap, "owner/tap");
    assert_eq!(
        availability(&item, Os::MacOs, Target::default_for(Os::Windows)),
        Availability::Unavailable(UnavailableReason::DifferentOs { source: Os::MacOs })
    );
    item.details = ItemDetails::Package {
        manager: PackageManager::Brew,
        kind: PackageKind::Formula,
    };
    assert_eq!(
        availability(
            &item,
            Os::MacOs,
            Target::default_for(Os::Linux(LinuxFamily::Other))
        ),
        Availability::Unavailable(UnavailableReason::NoPackageManager)
    );
    item.findings.push(Finding {
        line: None,
        kind: FindingKind::Unreadable,
    });
    assert_eq!(
        availability(&item, Os::MacOs, Target::default_for(Os::MacOs)),
        Availability::Unavailable(UnavailableReason::Unreadable)
    );
}
#[test]
fn target_defaults_validate_and_serialize() {
    for os in [
        Os::MacOs,
        Os::Windows,
        Os::Linux(LinuxFamily::Debian),
        Os::Linux(LinuxFamily::Fedora),
        Os::Linux(LinuxFamily::Arch),
        Os::Linux(LinuxFamily::Other),
    ] {
        let target = Target::default_for(os);
        assert!(target.validate().is_ok());
        assert_eq!(
            serde_json::from_value::<Target>(serde_json::to_value(target).unwrap()).unwrap(),
            target
        );
        for manager in [
            PackageManager::Brew,
            PackageManager::Winget,
            PackageManager::Scoop,
            PackageManager::Choco,
            PackageManager::Apt,
            PackageManager::Dnf,
            PackageManager::Pacman,
        ] {
            let allowed = Some(manager) == target.packages
                || (os == Os::Windows
                    && matches!(
                        manager,
                        PackageManager::Winget | PackageManager::Scoop | PackageManager::Choco
                    ));
            assert_eq!(
                Target {
                    os,
                    packages: Some(manager)
                }
                .validate()
                .is_ok(),
                allowed
            );
        }
        assert_eq!(
            Target { os, packages: None }.validate().is_ok(),
            os == Os::Linux(LinuxFamily::Other)
        );
    }
    assert_eq!(
        serde_json::to_value(Availability::Mapped {
            manager: PackageManager::Winget,
            name: "Git.Git".into()
        })
        .unwrap(),
        serde_json::json!({"mapped":{"manager":"winget","name":"Git.Git"}})
    );
}
