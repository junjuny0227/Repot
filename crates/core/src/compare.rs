//! Compare an opened Repot file with this machine (plan 12).
//!
//! Items are turned into the steps an export would run on the destination
//! (`steps_for_target`, so mapped names match the export) and this machine's scan into
//! its own steps (`steps_for`). "Present" therefore means what the restore script would
//! skip. Config files at the same place are compared with secrets masked on both sides.

use crate::export::{
    ConfigSource,
    paths::payload_path,
    payload::prepare_config,
    step::{Step, steps_for, steps_for_target},
};
use crate::{Item, ItemDetails, PackageManager, Target, os::Os};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Presence {
    /// The step the export would run is already satisfied here.
    Present,
    /// A config file exists at the same place with other contents.
    Differs,
    Missing,
}

/// What a step installs, normalized the way the restore script checks for it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Key {
    Package(PackageManager, crate::PackageKind, String),
    Tap(PackageManager, String),
    Runtime(crate::RuntimeTool, String, String),
    Global(crate::GlobalManager, String),
    Extension(crate::Editor, String),
    IdePlugin(crate::JetBrainsIde, String),
    Place(String),
}

fn key(step: &Step) -> Key {
    match step {
        Step::InstallPackage {
            manager,
            kind,
            name,
            ..
        } => {
            // Scoop's scan lists app names without the bucket ("firefox", not "extras/firefox").
            let name = if *manager == PackageManager::Scoop {
                name.rsplit('/').next().unwrap_or(name)
            } else {
                name
            };
            Key::Package(*manager, *kind, name.to_lowercase())
        }
        Step::AddTap { manager, name, .. } => Key::Tap(*manager, name.to_lowercase()),
        Step::InstallRuntime {
            tool,
            runtime,
            version,
            ..
        } => Key::Runtime(*tool, runtime.clone(), version.clone()),
        Step::InstallGlobal { manager, name } => Key::Global(*manager, name.to_lowercase()),
        Step::InstallExtension { editor, id } => Key::Extension(*editor, id.to_lowercase()),
        Step::InstallIdePlugin { ide, id, .. } => Key::IdePlugin(*ide, id.to_lowercase()),
        Step::CopyFile { dest, .. } | Step::GitClone { dest, .. } => Key::Place(payload_path(dest)),
    }
}

/// The step that decides an item's presence. A mapped Scoop package also adds its
/// bucket first; only the install itself is judged.
fn main_step(steps: Vec<Step>) -> Option<Step> {
    let install = steps.iter().position(|s| !matches!(s, Step::AddTap { .. }));
    match install {
        Some(index) => steps.into_iter().nth(index),
        None => steps.into_iter().next(),
    }
}

/// Presence of each item that is usable on `target`, judged against this machine.
/// Call it only when `target.os` is this machine's OS.
pub fn compare(
    items: &[Item],
    source: Os,
    target: Target,
    configs: &(impl ConfigSource + ?Sized),
    machine: &[Item],
    machine_os: Os,
    machine_configs: &(impl ConfigSource + ?Sized),
) -> HashMap<String, Presence> {
    let mut installed = HashSet::new();
    let mut places: HashMap<String, &Item> = HashMap::new();
    // A toolchain counts as here only with every component and target the file lists,
    // like the restore script's check.
    let mut extras: HashMap<Key, Extras> = HashMap::new();
    let machine_steps: Vec<(&Item, Vec<Step>)> = machine
        .iter()
        .map(|item| (item, steps_for(item, machine_os)))
        .collect();
    for (item, steps) in &machine_steps {
        for step in steps {
            let key = key(step);
            if let Step::InstallRuntime {
                components,
                targets,
                ..
            } = step
            {
                let here = extras.entry(key.clone()).or_default();
                here.0.extend(components);
                here.1.extend(targets);
            }
            if let (Key::Place(place), ItemDetails::ConfigFile { .. }) = (&key, &item.details) {
                places.insert(place.clone(), *item);
            }
            installed.insert(key);
        }
    }
    let mut result = HashMap::new();
    for item in items {
        let Some(step) = main_step(steps_for_target(item, source, target)) else {
            continue;
        };
        let key = key(&step);
        let presence = match (&item.details, &key) {
            (ItemDetails::ConfigFile { .. }, Key::Place(place)) => match places.get(place) {
                None => Presence::Missing,
                Some(here) => {
                    if same_contents(
                        item,
                        source,
                        target.os,
                        configs,
                        here,
                        machine_os,
                        machine_configs,
                    ) {
                        Presence::Present
                    } else {
                        Presence::Differs
                    }
                }
            },
            _ if installed.contains(&key) && has_extras(&step, extras.get(&key)) => {
                Presence::Present
            }
            _ => Presence::Missing,
        };
        result.insert(item.id.clone(), presence);
    }
    result
}

/// Components and targets of a toolchain on this machine.
type Extras<'a> = (BTreeSet<&'a String>, BTreeSet<&'a String>);

fn has_extras(step: &Step, here: Option<&Extras>) -> bool {
    let Step::InstallRuntime {
        components,
        targets,
        ..
    } = step
    else {
        return true;
    };
    let empty = Extras::default();
    let (have_components, have_targets) = here.unwrap_or(&empty);
    components.iter().all(|c| have_components.contains(c))
        && targets.iter().all(|t| have_targets.contains(t))
}

/// Both sides are masked with nothing revealed, so only non-secret text decides.
/// The file side is prepared for the destination like the export would write it.
fn same_contents(
    item: &Item,
    source: Os,
    target: Os,
    configs: &(impl ConfigSource + ?Sized),
    here: &Item,
    machine_os: Os,
    machine_configs: &(impl ConfigSource + ?Sized),
) -> bool {
    let (Some(theirs), Some(ours)) = (configs.read_config(item), machine_configs.read_config(here))
    else {
        return false;
    };
    let theirs = prepare_config(item, &theirs.text, &[], source, target).contents;
    let ours = prepare_config(here, &ours.text, &[], machine_os, machine_os).contents;
    theirs == ours
}
