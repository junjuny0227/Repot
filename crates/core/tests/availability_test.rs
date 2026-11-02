use repot_core::{
    availability::{Availability, UnavailableReason, availability, counterpart},
    model::*,
    os::{LinuxFamily, Os, Shell},
};
fn item(portability: Portability, findings: Vec<Finding>) -> Item {
    Item {
        id: "test:x".into(),
        provider: "test".into(),
        category: Category::Runtime,
        name: "x".into(),
        version: None,
        description: None,
        origin: Origin::Command("test".into()),
        portability,
        details: ItemDetails::Runtime {
            tool: RuntimeTool::Nvm,
            runtime: "node".into(),
            is_default: false,
        },
        findings,
    }
}
#[test]
fn counterpart_table() {
    use RuntimeTool::*;
    for os in [
        Os::Windows,
        Os::MacOs,
        Os::Linux(LinuxFamily::Debian),
        Os::Linux(LinuxFamily::Arch),
    ] {
        let windows = os == Os::Windows;
        for tool in [Nvm, NvmWindows] {
            assert_eq!(
                counterpart(tool, os),
                Some(if windows { NvmWindows } else { Nvm })
            );
        }
        for tool in [Pyenv, PyenvWin] {
            assert_eq!(
                counterpart(tool, os),
                Some(if windows { PyenvWin } else { Pyenv })
            );
        }
        for tool in [Fnm, Volta, Rustup] {
            assert_eq!(counterpart(tool, os), Some(tool));
        }
        assert_eq!(
            counterpart(Sdkman, os),
            if windows { None } else { Some(Sdkman) }
        );
    }
}
#[test]
fn same_os_requires_same_linux_family() {
    let it = item(Portability::SameOs, vec![]);
    let deb = Os::Linux(LinuxFamily::Debian);
    assert_eq!(availability(&it, deb, deb), Availability::Available);
    assert_eq!(
        availability(&it, deb, Os::Linux(LinuxFamily::Fedora)),
        Availability::Unavailable(UnavailableReason::DifferentOs { source: deb })
    );
    assert_eq!(
        availability(&it, Os::MacOs, Os::Windows),
        Availability::Unavailable(UnavailableReason::DifferentOs { source: Os::MacOs })
    );
}
#[test]
fn same_shell_and_mapped_and_any() {
    let zsh = item(Portability::SameShell(Shell::Zsh), vec![]);
    assert_eq!(
        availability(&zsh, Os::MacOs, Os::Linux(LinuxFamily::Debian)),
        Availability::Available
    );
    assert_eq!(
        availability(&zsh, Os::MacOs, Os::Windows),
        Availability::Unavailable(UnavailableReason::ShellUnsupported(Shell::Zsh))
    );
    assert_eq!(
        availability(
            &item(Portability::Mapped(RuntimeTool::Sdkman), vec![]),
            Os::MacOs,
            Os::Windows
        ),
        Availability::Unavailable(UnavailableReason::NoCounterpart(RuntimeTool::Sdkman))
    );
    for portability in [
        Portability::Any,
        Portability::Mapped(RuntimeTool::Nvm),
        Portability::SameShell(Shell::PowerShell),
    ] {
        assert_eq!(
            availability(&item(portability, vec![]), Os::MacOs, Os::Windows),
            Availability::Available
        );
    }
}
#[test]
fn unreadable_is_never_available() {
    for portability in [
        Portability::Any,
        Portability::SameOs,
        Portability::SameShell(Shell::Zsh),
        Portability::Mapped(RuntimeTool::Sdkman),
    ] {
        let it = item(
            portability,
            vec![Finding {
                line: None,
                kind: FindingKind::Unreadable,
            }],
        );
        assert_eq!(
            availability(&it, Os::MacOs, Os::Windows),
            Availability::Unavailable(UnavailableReason::Unreadable)
        );
    }
}
#[test]
fn variant_fields_serialize_camel_case() {
    let f = Finding {
        line: None,
        kind: FindingKind::Duplicate {
            other_provider: "choco".into(),
        },
    };
    assert_eq!(
        serde_json::to_string(&f).unwrap(),
        r#"{"line":null,"kind":{"duplicate":{"otherProvider":"choco"}}}"#
    );
    let it = item(Portability::Any, vec![f]);
    let json = serde_json::to_value(&it).unwrap();
    assert_eq!(json["details"]["runtime"]["isDefault"], false);
    assert_eq!(serde_json::from_value::<Item>(json).unwrap(), it);
    assert_eq!(
        serde_json::to_value(UnavailableReason::DifferentOs { source: Os::MacOs }).unwrap(),
        serde_json::json!({"differentOs":{"source":"macOs"}})
    );
}
