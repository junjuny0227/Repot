use crate::{
    model::{
        FindingKind, Item, ItemDetails, PackageKind, PackageManager, Portability, RuntimeTool,
        Target,
    },
    os::{Os, Shell},
    package_map::{self, Lookup},
};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Availability {
    Available,
    Mapped {
        manager: PackageManager,
        name: String,
    },
    Unavailable(UnavailableReason),
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum UnavailableReason {
    DifferentOs { source: Os },
    ShellUnsupported(Shell),
    NoCounterpart(RuntimeTool),
    NoPackageManager,
    NotInPackageMap { manager: PackageManager },
    NoPackageInManager { manager: PackageManager },
    Unreadable,
}
pub fn counterpart(tool: RuntimeTool, target: Os) -> Option<RuntimeTool> {
    use RuntimeTool::*;
    Some(match (tool, target) {
        (Nvm | NvmWindows, Os::Windows) => NvmWindows,
        (Nvm | NvmWindows, _) => Nvm,
        (Pyenv | PyenvWin, Os::Windows) => PyenvWin,
        (Pyenv | PyenvWin, _) => Pyenv,
        (Sdkman, Os::Windows) => return None,
        (other, _) => other,
    })
}
pub fn availability_map(
    items: &[Item],
    source: Os,
    target: Target,
) -> std::collections::HashMap<String, Availability> {
    items
        .iter()
        .map(|item| (item.id.clone(), availability(item, source, target)))
        .collect()
}

pub fn availability(item: &Item, source: Os, target: Target) -> Availability {
    let reason = if item
        .findings
        .iter()
        .any(|finding| finding.kind == FindingKind::Unreadable)
    {
        Some(UnavailableReason::Unreadable)
    } else {
        match item.portability {
            Portability::Package if source != target.os => {
                if let ItemDetails::Package { manager, kind, .. } = item.details {
                    if kind == PackageKind::Tap {
                        return Availability::Unavailable(UnavailableReason::DifferentOs {
                            source,
                        });
                    }
                    let Some(destination) = target.packages else {
                        return Availability::Unavailable(UnavailableReason::NoPackageManager);
                    };
                    return match package_map::embedded().lookup(
                        manager,
                        kind,
                        &item.name,
                        destination,
                    ) {
                        Lookup::Found { name, .. } => Availability::Mapped {
                            manager: destination,
                            name: name.into(),
                        },
                        Lookup::Absent => {
                            Availability::Unavailable(UnavailableReason::NoPackageInManager {
                                manager: destination,
                            })
                        }
                        Lookup::Missing => {
                            Availability::Unavailable(UnavailableReason::NotInPackageMap {
                                manager: destination,
                            })
                        }
                    };
                }
                Some(UnavailableReason::DifferentOs { source })
            }
            Portability::SameOs if source != target.os => {
                Some(UnavailableReason::DifferentOs { source })
            }
            Portability::SameShell(shell) if !target.os.supports_shell(shell) => {
                Some(UnavailableReason::ShellUnsupported(shell))
            }
            Portability::Mapped(tool) if counterpart(tool, target.os).is_none() => {
                Some(UnavailableReason::NoCounterpart(tool))
            }
            _ => None,
        }
    };
    reason.map_or(Availability::Available, Availability::Unavailable)
}
