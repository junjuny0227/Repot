use crate::{
    model::{FindingKind, Item, Portability, RuntimeTool},
    os::{Os, Shell},
};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Availability {
    Available,
    Unavailable(UnavailableReason),
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum UnavailableReason {
    DifferentOs { source: Os },
    ShellUnsupported(Shell),
    NoCounterpart(RuntimeTool),
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
    target: Os,
) -> std::collections::HashMap<String, Availability> {
    items
        .iter()
        .map(|item| (item.id.clone(), availability(item, source, target)))
        .collect()
}

pub fn availability(item: &Item, source: Os, target: Os) -> Availability {
    let reason = if item
        .findings
        .iter()
        .any(|finding| finding.kind == FindingKind::Unreadable)
    {
        Some(UnavailableReason::Unreadable)
    } else {
        match item.portability {
            Portability::SameOs if source != target => {
                Some(UnavailableReason::DifferentOs { source })
            }
            Portability::SameShell(shell) if !target.supports_shell(shell) => {
                Some(UnavailableReason::ShellUnsupported(shell))
            }
            Portability::Mapped(tool) if counterpart(tool, target).is_none() => {
                Some(UnavailableReason::NoCounterpart(tool))
            }
            _ => None,
        }
    };
    reason.map_or(Availability::Available, Availability::Unavailable)
}
