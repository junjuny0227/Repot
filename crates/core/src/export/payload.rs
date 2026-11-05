use super::paths::{os_label, payload_path};
use crate::{
    model::*,
    os::Os,
    providers::{git, ssh},
    secrets,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskedLine {
    pub file: String,
    pub line: u32,
    pub name: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportNote {
    CommentedOsLine {
        file: String,
        line: u32,
        key: String,
    },
    KeybindingsMayDiffer {
        file: String,
    },
    /// An SSH key the config points at; listed for the user to copy (plan 16).
    SshKey {
        file: String,
        line: u32,
        path: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedFile {
    pub path: String,
    pub contents: String,
    pub masked: Vec<MaskedLine>,
    pub notes: Vec<ExportNote>,
}

/// Prepare a configuration payload without reading or writing the filesystem.
///
/// # Panics
/// Panics if `item.details` is not `ItemDetails::ConfigFile`.
pub fn prepare_config(
    item: &Item,
    content: &str,
    reveal: &[u32],
    source: Os,
    target: Os,
) -> PreparedFile {
    let ItemDetails::ConfigFile {
        source: source_path,
        target: dest,
    } = &item.details
    else {
        panic!("prepare_config requires ItemDetails::ConfigFile");
    };
    let path = payload_path(dest);
    let mut masked: Vec<MaskedLine> = vec![];
    for secret in secrets::detect(content)
        .into_iter()
        .filter(|s| !reveal.contains(&s.line))
    {
        if let Some(previous) = masked.last_mut().filter(|m| m.line == secret.line) {
            if previous.name.is_none() {
                previous.name = secret.name;
            }
        } else {
            masked.push(MaskedLine {
                file: path.clone(),
                line: secret.line,
                name: secret.name,
            });
        }
    }
    let mut contents = secrets::mask(content, reveal);
    let mut notes = vec![];
    let ssh = item.provider == "ssh";
    // Key paths come from every key line, including the ones commented out below.
    let keys = if ssh {
        ssh::key_paths(&contents)
    } else {
        vec![]
    };
    if !source.same_kind(target) {
        // Source paths can belong to a different OS than the machine doing the export.
        let source_path = source_path.to_string_lossy().replace('\\', "/");
        let filename = source_path.rsplit('/').next().unwrap_or("");
        let label = os_label(target);
        let lines: Vec<(u32, String, String)> =
            if item.provider == "git" && matches!(filename, ".gitconfig" | "config") {
                git::os_specific_lines(&contents)
                    .into_iter()
                    .filter_map(|finding| {
                        let FindingKind::OsSpecificLine { key } = finding.kind else {
                            return None;
                        };
                        let suggestion = match key.as_str() {
                            "credential.helper" => Some(match target {
                                Os::MacOs => "osxkeychain",
                                Os::Windows => "manager",
                                Os::Linux(_) => "libsecret",
                            }),
                            "core.autocrlf" => Some(if target == Os::Windows {
                                "true"
                            } else {
                                "input"
                            }),
                            _ => None,
                        };
                        let guidance = match suggestion {
                            Some(value) => format!("on {label} try: {value}"),
                            None => format!("adjust it for {label}"),
                        };
                        Some((finding.line?, key, guidance))
                    })
                    .collect()
            } else if ssh {
                ssh::lines_to_comment(&contents, target)
                    .into_iter()
                    .map(|(line, key)| (line, key, format!("adjust it for {label}")))
                    .collect()
            } else {
                vec![]
            };
        if !lines.is_empty() {
            contents = comment_out(&contents, &lines, &path, &mut notes);
        }
        if filename == "keybindings.json" && ((source == Os::MacOs) != (target == Os::MacOs)) {
            notes.push(ExportNote::KeybindingsMayDiffer { file: path.clone() });
        }
    }
    notes.extend(keys.into_iter().map(|(line, key)| ExportNote::SshKey {
        file: path.clone(),
        line,
        path: key,
    }));
    PreparedFile {
        path,
        contents,
        masked,
        notes,
    }
}

/// Comment out `(line, key, guidance)` lines in place, so line numbers stay the same.
fn comment_out(
    contents: &str,
    lines: &[(u32, String, String)],
    path: &str,
    notes: &mut Vec<ExportNote>,
) -> String {
    contents
        .split_inclusive('\n')
        .enumerate()
        .map(|(index, raw)| {
            let line = (index + 1) as u32;
            let Some((_, key, guidance)) = lines.iter().find(|(l, _, _)| *l == line) else {
                return raw.to_owned();
            };
            let (body, ending) = if let Some(body) = raw.strip_suffix("\r\n") {
                (body, "\r\n")
            } else if let Some(body) = raw.strip_suffix('\n') {
                (body, "\n")
            } else {
                (raw, "")
            };
            let indent = &body[..body.len() - body.trim_start().len()];
            notes.push(ExportNote::CommentedOsLine {
                file: path.into(),
                line,
                key: key.clone(),
            });
            format!(
                "{indent}# [repot] {}  # {key} is OS-specific; {guidance}{ending}",
                body.trim()
            )
        })
        .collect()
}
