//! Explicit, offline package correspondences. Missing keys and verified absence stay distinct.
use crate::{
    export::step::is_safe_value,
    model::{PackageKind, PackageManager},
};
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, Visitor},
};
use std::{collections::HashMap, fmt, sync::OnceLock};

#[derive(Debug, PartialEq)]
pub enum Lookup<'a> {
    Found { kind: PackageKind, name: &'a str },
    Absent,
    Missing,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
enum ManagerKey {
    Brew,
    BrewCask,
    Winget,
    Scoop,
    Choco,
    Apt,
    Dnf,
    Pacman,
}
impl ManagerKey {
    fn parts(self) -> (PackageManager, PackageKind) {
        use PackageKind::*;
        match self {
            Self::Brew => (PackageManager::Brew, Formula),
            Self::BrewCask => (PackageManager::Brew, Cask),
            Self::Winget => (PackageManager::Winget, Package),
            Self::Scoop => (PackageManager::Scoop, Package),
            Self::Choco => (PackageManager::Choco, Package),
            Self::Apt => (PackageManager::Apt, Package),
            Self::Dnf => (PackageManager::Dnf, Package),
            Self::Pacman => (PackageManager::Pacman, Package),
        }
    }
}
type Entry = HashMap<ManagerKey, Option<String>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Data {
    version: u32,
    #[serde(deserialize_with = "ordered_packages")]
    packages: Vec<Entry>,
}
fn ordered_packages<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Entry>, D::Error> {
    struct Packages;
    impl<'de> Visitor<'de> for Packages {
        type Value = Vec<Entry>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("sorted package ids")
        }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut previous = String::new();
            let mut entries = vec![];
            while let Some((id, entry)) = map.next_entry::<String, Entry>()? {
                if id.is_empty()
                    || !id
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                    || id <= previous
                {
                    return Err(de::Error::custom(
                        "invalid, duplicate or unsorted package id",
                    ));
                }
                previous = id;
                entries.push(entry);
            }
            Ok(entries)
        }
    }
    d.deserialize_map(Packages)
}

pub struct PackageMap {
    entries: Vec<Entry>,
    index: HashMap<(PackageManager, PackageKind, String), usize>,
}
fn normalized(manager: PackageManager, name: &str) -> String {
    let name = if manager == PackageManager::Scoop {
        name.strip_prefix("main/").unwrap_or(name)
    } else {
        name
    };
    name.to_ascii_lowercase()
}
impl PackageMap {
    pub fn parse(json: &str) -> Result<Self, String> {
        let data: Data = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if data.version != 1 {
            return Err("unsupported package map version".into());
        }
        let mut index = HashMap::new();
        for (i, entry) in data.packages.iter().enumerate() {
            for (&key, name) in entry {
                let Some(name) = name else { continue };
                if !is_safe_value(name) {
                    return Err("invalid package name".into());
                }
                if key == ManagerKey::Scoop {
                    let (bucket, app) = name.split_once('/').unwrap_or(("main", name));
                    if !matches!(
                        bucket,
                        "main" | "extras" | "versions" | "java" | "nerd-fonts"
                    ) || app.contains('/')
                        || app.is_empty()
                    {
                        return Err("invalid Scoop bucket/name".into());
                    }
                }
                let (manager, kind) = key.parts();
                if index
                    .insert((manager, kind, normalized(manager, name)), i)
                    .is_some()
                {
                    return Err("duplicate package name (case insensitive)".into());
                }
            }
        }
        Ok(Self {
            entries: data.packages,
            index,
        })
    }
    pub fn lookup(
        &self,
        manager: PackageManager,
        kind: PackageKind,
        name: &str,
        target: PackageManager,
    ) -> Lookup<'_> {
        let Some(&i) = self.index.get(&(manager, kind, normalized(manager, name))) else {
            return Lookup::Missing;
        };
        let entry = &self.entries[i];
        // A package can have only one destination kind per manager. Prefer formula if both exist.
        let keys: &[ManagerKey] = match target {
            PackageManager::Brew => &[ManagerKey::Brew, ManagerKey::BrewCask],
            PackageManager::Winget => &[ManagerKey::Winget],
            PackageManager::Scoop => &[ManagerKey::Scoop],
            PackageManager::Choco => &[ManagerKey::Choco],
            PackageManager::Apt => &[ManagerKey::Apt],
            PackageManager::Dnf => &[ManagerKey::Dnf],
            PackageManager::Pacman => &[ManagerKey::Pacman],
        };
        for key in keys {
            if let Some(Some(name)) = entry.get(key) {
                return Lookup::Found {
                    kind: key.parts().1,
                    name,
                };
            }
        }
        if keys.iter().any(|key| entry.contains_key(key)) {
            Lookup::Absent
        } else {
            Lookup::Missing
        }
    }
}
pub fn embedded() -> &'static PackageMap {
    static MAP: OnceLock<PackageMap> = OnceLock::new();
    MAP.get_or_init(|| {
        PackageMap::parse(include_str!("../data/package-map.json"))
            .expect("validated embedded package map")
    })
}
