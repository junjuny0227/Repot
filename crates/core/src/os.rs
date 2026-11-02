use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Os {
    MacOs,
    Windows,
    Linux(LinuxFamily),
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum LinuxFamily {
    Debian,
    Fedora,
    Arch,
    Other,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
    PowerShell,
}

pub fn parse_os_release(content: &str) -> LinuxFamily {
    let value = |key: &str| {
        content
            .lines()
            .filter_map(|line| line.split_once('='))
            .find(|(name, _)| name.trim() == key)
            .map(|(_, value)| value.trim().trim_matches(['\'', '"']))
            .unwrap_or("")
    };
    fn family(id: &str) -> Option<LinuxFamily> {
        match id {
            "debian" | "ubuntu" => Some(LinuxFamily::Debian),
            "fedora" | "rhel" | "centos" => Some(LinuxFamily::Fedora),
            "arch" => Some(LinuxFamily::Arch),
            _ => None,
        }
    }
    family(value("ID"))
        .or_else(|| value("ID_LIKE").split_whitespace().find_map(family))
        .unwrap_or(LinuxFamily::Other)
}

pub fn current_os() -> Os {
    if cfg!(target_os = "macos") {
        Os::MacOs
    } else if cfg!(target_os = "windows") {
        Os::Windows
    } else {
        Os::Linux(parse_os_release(
            &std::fs::read_to_string("/etc/os-release").unwrap_or_default(),
        ))
    }
}
impl Os {
    /// Compare OS kinds without distinguishing Linux distribution families.
    pub fn same_kind(self, other: Os) -> bool {
        std::mem::discriminant(&self) == std::mem::discriminant(&other)
    }

    pub fn supports_shell(self, shell: Shell) -> bool {
        shell == Shell::PowerShell || self != Self::Windows
    }
}
