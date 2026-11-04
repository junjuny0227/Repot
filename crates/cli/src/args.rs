//! Command-line arguments. Parsing only; `run` decides what they mean.

use clap::{Parser, Subcommand, ValueEnum};
use repot_core::os::{LinuxFamily, Os};
use repot_core::{PackageManager, export::Format};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "repot",
    version,
    about = "Move your development environment to a new machine.",
    long_about = "Move your development environment to a new machine.\n\n\
        Repot only reads this machine. It writes one export folder with a restore script \
        you can read and run yourself, and it never uses the network."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Scan this machine and list what Repot found.
    Scan {
        /// Print the scan result as JSON (the shape may change while Repot is 0.x).
        #[arg(long)]
        json: bool,
    },
    /// Compare a Repot file (repot.json) with this machine.
    Compare {
        /// The repot.json from an export.
        file: PathBuf,
        /// Print each item's state as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Export a restore script, or native manifests, and a new repot.json.
    Export(ExportArgs),
}

#[derive(clap::Args, Debug)]
pub struct ExportArgs {
    /// Export from this Repot file instead of scanning this machine.
    pub file: Option<PathBuf>,
    /// Destination OS. Defaults to this machine's OS.
    #[arg(long, value_enum)]
    pub to: Option<TargetOs>,
    /// Package manager on the destination (Windows: winget, scoop or choco).
    #[arg(long, value_enum)]
    pub packages: Option<Manager>,
    /// What to write.
    #[arg(long, value_enum, default_value_t = FormatArg::Script)]
    pub format: FormatArg,
    /// Compare the file with this machine and select only what it is missing.
    #[arg(long)]
    pub only_missing: bool,
    /// Also select this item id (repeatable). Ids are shown by `scan` and `compare`.
    #[arg(long, value_name = "ID")]
    pub include: Vec<String>,
    /// Leave out this item id (repeatable).
    #[arg(long, value_name = "ID")]
    pub exclude: Vec<String>,
    /// Folder to create the export folder in.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub out: PathBuf,
    /// Show what would be written without writing anything.
    #[arg(long)]
    pub dry_run: bool,
    /// Print the result as JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum TargetOs {
    Macos,
    Windows,
    LinuxDebian,
    LinuxFedora,
    LinuxArch,
    LinuxOther,
}
impl TargetOs {
    pub fn os(self) -> Os {
        match self {
            Self::Macos => Os::MacOs,
            Self::Windows => Os::Windows,
            Self::LinuxDebian => Os::Linux(LinuxFamily::Debian),
            Self::LinuxFedora => Os::Linux(LinuxFamily::Fedora),
            Self::LinuxArch => Os::Linux(LinuxFamily::Arch),
            Self::LinuxOther => Os::Linux(LinuxFamily::Other),
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum Manager {
    Brew,
    Winget,
    Scoop,
    Choco,
    Apt,
    Dnf,
    Pacman,
}
impl Manager {
    pub fn manager(self) -> PackageManager {
        match self {
            Self::Brew => PackageManager::Brew,
            Self::Winget => PackageManager::Winget,
            Self::Scoop => PackageManager::Scoop,
            Self::Choco => PackageManager::Choco,
            Self::Apt => PackageManager::Apt,
            Self::Dnf => PackageManager::Dnf,
            Self::Pacman => PackageManager::Pacman,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum FormatArg {
    /// One restore script for the destination shell.
    Script,
    /// Native manifests (Brewfile, winget JSON, extension lists and so on).
    Manifest,
}
impl FormatArg {
    pub fn format(self) -> Format {
        match self {
            Self::Script => Format::Script,
            Self::Manifest => Format::Manifest,
        }
    }
}
