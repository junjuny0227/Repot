//! What the review and export steps work on, shared by the desktop app and the CLI
//! (plan 15): a scan of this machine or an opened Repot file, plus an optional scan of
//! this machine to compare the file with.

use crate::availability::{Availability, availability_map};
use crate::compare::{Presence, compare};
use crate::env::Env;
use crate::export::{ExportBundle, ExportMeta, ExportRequest, Format, build_export};
use crate::os::Os;
use crate::scan::ScanResult;
use crate::snapshot::{self, Snapshot, SnapshotError};
use crate::{Selection, Target};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;

/// Where config-file contents come from.
#[derive(Debug, Clone)]
pub enum Configs {
    /// Read config files from this machine.
    Live,
    /// Use the contents stored in an opened Repot file.
    Snapshot(Arc<Snapshot>),
}

/// Source OS, items and config contents always come from the same place.
#[derive(Debug, Clone)]
pub struct Session {
    pub source: Os,
    pub scan: ScanResult,
    pub configs: Configs,
    /// This machine's scan, taken to compare an opened Repot file with it.
    pub machine: Option<ScanResult>,
}

/// Facts about an opened file that are not part of the session itself.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenedInfo {
    pub created_at: String,
    pub repot_version: String,
    pub skipped_items: usize,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum OpenError {
    /// Missing or unreadable.
    #[error("could not read the file")]
    ReadFailed,
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}

static NO_IDS: BTreeSet<String> = BTreeSet::new();

impl Session {
    /// A scan of this machine.
    pub fn from_scan(source: Os, scan: ScanResult) -> Self {
        Self {
            source,
            scan,
            configs: Configs::Live,
            machine: None,
        }
    }

    /// Open a Repot file. Oversized files are refused before they are read.
    pub fn open(env: &dyn Env, path: &Path) -> Result<(Self, OpenedInfo), OpenError> {
        if env
            .file_len(path)
            .is_ok_and(|len| len > snapshot::MAX_BYTES as u64)
        {
            return Err(SnapshotError::TooLarge.into());
        }
        let text = env.read_file(path).map_err(|_| OpenError::ReadFailed)?;
        let opened = snapshot::parse_snapshot(&text)?;
        let snapshot = Arc::new(opened.snapshot);
        let info = OpenedInfo {
            created_at: snapshot.created_at.clone(),
            repot_version: snapshot.repot_version.clone(),
            skipped_items: opened.skipped_items,
        };
        let session = Self {
            source: snapshot.source,
            scan: ScanResult {
                items: snapshot.items.clone(),
                providers: vec![],
            },
            configs: Configs::Snapshot(snapshot),
            machine: None,
        };
        Ok((session, info))
    }

    /// Items the export that wrote the opened file could not include (plan 13).
    pub fn not_exported(&self) -> &BTreeSet<String> {
        match &self.configs {
            Configs::Live => &NO_IDS,
            Configs::Snapshot(snapshot) => &snapshot.not_exported,
        }
    }

    pub fn availability(&self, target: Target) -> HashMap<String, Availability> {
        availability_map(&self.scan.items, self.source, target)
    }

    /// Presence of each item on this machine. Empty unless this machine was scanned
    /// for comparison and `target` is this machine's OS (`this_os`).
    pub fn presence(
        &self,
        env: &dyn Env,
        this_os: Os,
        target: Target,
    ) -> HashMap<String, Presence> {
        let Some(machine) = self.machine.as_ref().filter(|_| target.os == this_os) else {
            return HashMap::new();
        };
        let items = &self.scan.items;
        match &self.configs {
            Configs::Live => compare(
                items,
                self.source,
                target,
                env,
                &machine.items,
                this_os,
                env,
            ),
            Configs::Snapshot(snapshot) => compare(
                items,
                self.source,
                target,
                snapshot.as_ref(),
                &machine.items,
                this_os,
                env,
            ),
        }
    }

    /// The export for `selection`, in memory. Writing it is `export::write_export`.
    pub fn preview(
        &self,
        env: &dyn Env,
        selection: &Selection,
        target: Target,
        format: Format,
        meta: ExportMeta,
    ) -> Result<ExportBundle, &'static str> {
        target.validate()?;
        let request = ExportRequest {
            items: &self.scan.items,
            selection,
            source: self.source,
            target,
            format,
            meta,
        };
        Ok(match &self.configs {
            Configs::Live => build_export(env, &request),
            Configs::Snapshot(snapshot) => build_export(snapshot.as_ref(), &request),
        })
    }

    /// The items selected at first, in item order (spec §11; plans 12 and 13):
    /// everything usable on `target`, minus items the opened file left out of its
    /// export; after a comparison (`presence`), only what this machine is missing.
    pub fn default_selection(
        &self,
        target: Target,
        presence: Option<&HashMap<String, Presence>>,
    ) -> Vec<String> {
        let avail = self.availability(target);
        let not_exported = self.not_exported();
        self.scan
            .items
            .iter()
            .filter(|item| {
                !matches!(
                    avail.get(&item.id),
                    Some(Availability::Unavailable(_)) | None
                )
            })
            .filter(|item| !not_exported.contains(&item.id))
            .filter(|item| presence.is_none_or(|p| p.get(&item.id) == Some(&Presence::Missing)))
            .map(|item| item.id.clone())
            .collect()
    }
}
