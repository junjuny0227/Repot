//! JetBrains IDE facts shared by the scan and the restore scripts (plan 14).

use crate::model::JetBrainsIde;

impl JetBrainsIde {
    pub const ALL: [JetBrainsIde; 12] = [
        Self::IntelliJIdea,
        Self::IdeaIc,
        Self::WebStorm,
        Self::PyCharm,
        Self::PyCharmCe,
        Self::GoLand,
        Self::RustRover,
        Self::CLion,
        Self::PhpStorm,
        Self::RubyMine,
        Self::DataGrip,
        Self::Rider,
    ];
    /// Settings-folder prefix: `IntelliJIdea2026.1`.
    pub fn prefix(self) -> &'static str {
        match self {
            Self::IntelliJIdea => "IntelliJIdea",
            Self::IdeaIc => "IdeaIC",
            Self::WebStorm => "WebStorm",
            Self::PyCharm => "PyCharm",
            Self::PyCharmCe => "PyCharmCE",
            Self::GoLand => "GoLand",
            Self::RustRover => "RustRover",
            Self::CLion => "CLion",
            Self::PhpStorm => "PhpStorm",
            Self::RubyMine => "RubyMine",
            Self::DataGrip => "DataGrip",
            Self::Rider => "Rider",
        }
    }
    /// Command-line launcher name (Toolbox scripts and the app's own launcher).
    pub fn command(self) -> &'static str {
        match self {
            Self::IntelliJIdea | Self::IdeaIc => "idea",
            Self::WebStorm => "webstorm",
            Self::PyCharm | Self::PyCharmCe => "pycharm",
            Self::GoLand => "goland",
            Self::RustRover => "rustrover",
            Self::CLion => "clion",
            Self::PhpStorm => "phpstorm",
            Self::RubyMine => "rubymine",
            Self::DataGrip => "datagrip",
            Self::Rider => "rider",
        }
    }
    /// macOS app bundle names, checked in `/Applications` and `~/Applications`.
    pub fn mac_apps(self) -> &'static [&'static str] {
        match self {
            Self::IntelliJIdea => &["IntelliJ IDEA.app", "IntelliJ IDEA Ultimate.app"],
            Self::IdeaIc => &[
                "IntelliJ IDEA CE.app",
                "IntelliJ IDEA Community Edition.app",
            ],
            Self::PyCharm => &["PyCharm.app", "PyCharm Professional Edition.app"],
            Self::PyCharmCe => &["PyCharm CE.app", "PyCharm Community Edition.app"],
            Self::WebStorm => &["WebStorm.app"],
            Self::GoLand => &["GoLand.app"],
            Self::RustRover => &["RustRover.app"],
            Self::CLion => &["CLion.app"],
            Self::PhpStorm => &["PhpStorm.app"],
            Self::RubyMine => &["RubyMine.app"],
            Self::DataGrip => &["DataGrip.app"],
            Self::Rider => &["Rider.app"],
        }
    }
    /// Windows install folder patterns under `Program Files\JetBrains` and
    /// `%LOCALAPPDATA%\Programs`.
    pub fn windows_dirs(self) -> &'static [&'static str] {
        match self {
            Self::IntelliJIdea => &[
                "IntelliJ IDEA 20*",
                "IntelliJ IDEA Ultimate*",
                "IntelliJ IDEA",
            ],
            Self::IdeaIc => &["IntelliJ IDEA Community Edition*"],
            Self::PyCharm => &["PyCharm 20*", "PyCharm Professional*", "PyCharm"],
            Self::PyCharmCe => &["PyCharm Community Edition*"],
            Self::WebStorm => &["WebStorm*"],
            Self::GoLand => &["GoLand*"],
            Self::RustRover => &["RustRover*"],
            Self::CLion => &["CLion*"],
            Self::PhpStorm => &["PhpStorm*"],
            Self::RubyMine => &["RubyMine*"],
            Self::DataGrip => &["DataGrip*"],
            Self::Rider => &["JetBrains Rider*", "Rider*"],
        }
    }
    /// Product name shown in scripts (`IntelliJ IDEA not found`).
    pub fn product(self) -> &'static str {
        match self {
            Self::IntelliJIdea => "IntelliJ IDEA",
            Self::IdeaIc => "IntelliJ IDEA CE",
            Self::WebStorm => "WebStorm",
            Self::PyCharm => "PyCharm",
            Self::PyCharmCe => "PyCharm CE",
            Self::GoLand => "GoLand",
            Self::RustRover => "RustRover",
            Self::CLion => "CLion",
            Self::PhpStorm => "PhpStorm",
            Self::RubyMine => "RubyMine",
            Self::DataGrip => "DataGrip",
            Self::Rider => "Rider",
        }
    }
    /// Folder name `<prefix><year>.<n>` split into IDE and version.
    pub fn parse_folder(name: &str) -> Option<(Self, String)> {
        // Longest prefix first, so `PyCharmCE2026.1` is not read as PyCharm "CE2026.1".
        let mut all = Self::ALL;
        all.sort_by_key(|ide| std::cmp::Reverse(ide.prefix().len()));
        all.into_iter().find_map(|ide| {
            let version = name.strip_prefix(ide.prefix())?;
            is_version(version).then(|| (ide, version.to_owned()))
        })
    }
}

/// `2026.1` (year, dot, release); nothing else is accepted as a folder version.
pub fn is_version(version: &str) -> bool {
    let mut parts = version.split('.');
    let (Some(year), Some(n), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    year.len() == 4
        && !n.is_empty()
        && n.len() <= 2
        && year.bytes().chain(n.bytes()).all(|b| b.is_ascii_digit())
}

/// Numeric order of two `is_version` strings.
pub fn version_key(version: &str) -> (u32, u32) {
    let mut parts = version.split('.').map(|p| p.parse().unwrap_or(0));
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0))
}

/// Where restore scripts send people who do not have the IDE yet.
pub const INSTALL_URL: &str = "https://www.jetbrains.com/toolbox-app/";
